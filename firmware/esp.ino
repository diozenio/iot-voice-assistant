/*
 * ESP32 - Microfone INMP441 -> Wi-Fi (WebSocket) -> servidor TS
 * Botão alterna entre gravar/parar. LED aceso = enviando áudio.
 * Formato: PCM 16 kHz, 16 bits, mono.
 */

 #include <WiFi.h>
 #include <WebSocketsClient.h>
 #include <driver/i2s_std.h>
 
 // ---- Configuração ----
 const char* WIFI_SSID = "SUA_REDE";
 const char* WIFI_PASS = "SUA_SENHA";
 const char* SERVER_IP = "192.168.0.10";   // IP do computador na rede local
 const uint16_t SERVER_PORT = 3000;
 
 #define PIN_BOTAO   27
 #define PIN_LED     26
 #define I2S_SCK     32
 #define I2S_WS      25
 #define I2S_SD      33
 #define SAMPLE_RATE 16000
 #define BUF_LEN     512
 #define GANHO       4.0f
 #define HP_CORTE    150.0f   // corta o ronco de baixa frequencia que abafava a fala

 i2s_chan_handle_t rx_chan;
 WebSocketsClient ws;

 bool conectado = false;
 bool gravando = false;

 float hp_b0, hp_b1, hp_b2, hp_a1, hp_a2;
 float hp_x1 = 0, hp_x2 = 0, hp_y1 = 0, hp_y2 = 0;

 bool botaoAnterior = HIGH;
 unsigned long ultimoToque = 0;
 const unsigned long DEBOUNCE_MS = 50;
 
 void eventoWS(WStype_t tipo, uint8_t* payload, size_t tam) {
   switch (tipo) {
     case WStype_CONNECTED:
       conectado = true;
       Serial.println("WebSocket conectado");
       break;
     case WStype_DISCONNECTED:
       conectado = false;
       if (gravando) {
         gravando = false;
         digitalWrite(PIN_LED, LOW);
       }
       Serial.println("WebSocket desconectado");
       break;
     case WStype_TEXT:
       Serial.printf("Servidor: %s\n", (char*)payload);
       break;
     default:
       break;
   }
 }
 
 // Passa-alta biquad (Butterworth, Q=0.707) em HP_CORTE
 void iniciarFiltro() {
   float w = 2.0f * PI * HP_CORTE / SAMPLE_RATE;
   float c = cosf(w);
   float a = sinf(w) / (2.0f * 0.707f);
   float a0 = 1.0f + a;

   hp_b0 = ((1.0f + c) / 2.0f) / a0;
   hp_b1 = (-(1.0f + c)) / a0;
   hp_b2 = hp_b0;
   hp_a1 = (-2.0f * c) / a0;
   hp_a2 = (1.0f - a) / a0;
 }

 void zerarFiltro() {
   hp_x1 = hp_x2 = hp_y1 = hp_y2 = 0;
 }

 float filtrarHP(float x) {
   float y = hp_b0 * x + hp_b1 * hp_x1 + hp_b2 * hp_x2 - hp_a1 * hp_y1 - hp_a2 * hp_y2;
   hp_x2 = hp_x1; hp_x1 = x;
   hp_y2 = hp_y1; hp_y1 = y;
   return y;
 }

 void iniciarMicrofone() {
   i2s_chan_config_t chan_cfg = I2S_CHANNEL_DEFAULT_CONFIG(I2S_NUM_0, I2S_ROLE_MASTER);
   // ~256 ms de folga: absorve o tempo do envio WiFi sem descartar amostras
   chan_cfg.dma_desc_num = 8;
   chan_cfg.dma_frame_num = 512;
   i2s_new_channel(&chan_cfg, NULL, &rx_chan);
 
   i2s_std_config_t std_cfg = {
     .clk_cfg = I2S_STD_CLK_DEFAULT_CONFIG(SAMPLE_RATE),
     .slot_cfg = I2S_STD_PHILIPS_SLOT_DEFAULT_CONFIG(I2S_DATA_BIT_WIDTH_32BIT, I2S_SLOT_MODE_MONO),
     .gpio_cfg = {
       .mclk = I2S_GPIO_UNUSED,
       .bclk = (gpio_num_t)I2S_SCK,
       .ws   = (gpio_num_t)I2S_WS,
       .dout = I2S_GPIO_UNUSED,
       .din  = (gpio_num_t)I2S_SD,
       .invert_flags = { false, false, false },
     },
   };
   std_cfg.slot_cfg.slot_mask = I2S_STD_SLOT_LEFT;
 
   i2s_channel_init_std_mode(rx_chan, &std_cfg);
   i2s_channel_enable(rx_chan);
 }
 
 void verificarBotao() {
   bool leitura = digitalRead(PIN_BOTAO);
   if (botaoAnterior == HIGH && leitura == LOW &&
       millis() - ultimoToque > DEBOUNCE_MS) {
     ultimoToque = millis();
 
     if (!gravando) {
       if (!conectado) {
         Serial.println("Sem conexão com o servidor, ignorando.");
       } else {
         gravando = true;
         zerarFiltro();
         ws.sendTXT("start");
         digitalWrite(PIN_LED, HIGH);
         Serial.println(">>> Gravando");
       }
     } else {
       gravando = false;
       ws.sendTXT("stop");
       digitalWrite(PIN_LED, LOW);
       Serial.println(">>> Parado, enviando para transcrição");
     }
   }
   botaoAnterior = leitura;
 }
 
 void setup() {
   Serial.begin(115200);
 
   pinMode(PIN_BOTAO, INPUT_PULLUP);
   pinMode(PIN_LED, OUTPUT);
   digitalWrite(PIN_LED, LOW);
 
   WiFi.mode(WIFI_STA);
   WiFi.setSleep(false);   // menos latência e menos engasgos no envio
   WiFi.begin(WIFI_SSID, WIFI_PASS);
   Serial.print("Conectando ao Wi-Fi");
   while (WiFi.status() != WL_CONNECTED) {
     delay(500);
     Serial.print(".");
   }
   Serial.printf("\nWi-Fi OK, IP: %s\n", WiFi.localIP().toString().c_str());
 
   ws.begin(SERVER_IP, SERVER_PORT, "/");
   ws.onEvent(eventoWS);
   ws.setReconnectInterval(3000);
 
   iniciarFiltro();
   iniciarMicrofone();
 }
 
 void loop() {
   ws.loop();
   verificarBotao();
 
   static int32_t amostras[BUF_LEN];
   static int16_t saida[BUF_LEN];
   size_t bytes_lidos = 0;
 
   // Sempre lê o microfone para o buffer I2S não acumular áudio antigo
   i2s_channel_read(rx_chan, amostras, sizeof(amostras), &bytes_lidos, 100);
   int n = bytes_lidos / sizeof(int32_t);
 
   if (!gravando || !conectado || n == 0) return;
 
   for (int i = 0; i < n; i++) {
     int32_t s = amostras[i] >> 8;                     // 24 bits
     float x = filtrarHP(s / 256.0f) * GANHO;          // 24 -> 16 bits, sem o ronco
     if (x > 32767) x = 32767;
     if (x < -32768) x = -32768;
     saida[i] = (int16_t)x;
   }
 
   ws.sendBIN((uint8_t*)saida, n * sizeof(int16_t));
 }