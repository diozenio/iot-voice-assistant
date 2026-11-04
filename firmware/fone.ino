/*
 * ESP32 - Microfone INMP441 (I2S) -> fone Bluetooth (A2DP)
 * Botão liga/desliga a transmissão; LED indica que está ativa.
 *
 * Microfone: VDD->3V3 | GND->GND | L/R->GND | SCK->32 | WS->25 | SD->33
 * Botão:     GPIO 27 <-> GND
 * LED:       GPIO 26 -> resistor 220R -> LED -> GND
 */

 #include "AudioTools.h"
 #include "AudioTools/Communication/A2DPStream.h"
 #include <driver/i2s_std.h>
 
 // ---- Configuração ----
 const char*  NOME_FONE   = "JBL Wave Buds 2";
 const float  GANHO       = 4.0;     // volume constante (ajuste a gosto)
 
 #define PIN_BOTAO   27
 #define PIN_LED     26
 #define I2S_SCK     32
 #define I2S_WS      25
 #define I2S_SD      33
 #define SAMPLE_RATE 44100           // A2DP exige 44100 Hz
 #define BUF_LEN     256
 
 A2DPStream a2dp;
 i2s_chan_handle_t rx_chan;
 
 bool transmitindo = false;
 float dc = 0;
 
 // Controle do botão (debounce)
 bool botaoAnterior = HIGH;
 unsigned long ultimoToque = 0;
 const unsigned long DEBOUNCE_MS = 50;
 
 void iniciarMicrofone() {
   i2s_chan_config_t chan_cfg = I2S_CHANNEL_DEFAULT_CONFIG(I2S_NUM_0, I2S_ROLE_MASTER);
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
   std_cfg.slot_cfg.slot_mask = I2S_STD_SLOT_LEFT;   // L/R no GND
 
   i2s_channel_init_std_mode(rx_chan, &std_cfg);
   i2s_channel_enable(rx_chan);
 }
 
 void verificarBotao() {
   bool leitura = digitalRead(PIN_BOTAO);
 
   // Detecta a borda de descida (apertou) com debounce
   if (botaoAnterior == HIGH && leitura == LOW &&
       millis() - ultimoToque > DEBOUNCE_MS) {
     ultimoToque = millis();
     transmitindo = !transmitindo;
     digitalWrite(PIN_LED, transmitindo ? HIGH : LOW);
     Serial.println(transmitindo ? ">>> Transmitindo" : ">>> Parado");
   }
   botaoAnterior = leitura;
 }
 
 void setup() {
   Serial.begin(115200);
   delay(1000);
   Serial.println("\n=== ESP32 - Microfone -> Bluetooth (botao liga/desliga) ===");
 
   pinMode(PIN_BOTAO, INPUT_PULLUP);
   pinMode(PIN_LED, OUTPUT);
   digitalWrite(PIN_LED, LOW);
 
   auto cfg = a2dp.defaultConfig(TX_MODE);   // 44100 Hz, estéreo, 16 bits
   cfg.name = NOME_FONE;
   cfg.auto_reconnect = true;
   a2dp.begin(cfg);
 
   iniciarMicrofone();
   Serial.printf("Procurando '%s'...\n", NOME_FONE);
 }
 
 void loop() {
   verificarBotao();
 
   int32_t amostras[BUF_LEN];
   int16_t estereo[BUF_LEN * 2];
   size_t bytes_lidos = 0;
 
   // Sempre lê o microfone, para o buffer I2S não acumular dados antigos
   i2s_channel_read(rx_chan, amostras, sizeof(amostras), &bytes_lidos, 100);
   int n = bytes_lidos / sizeof(int32_t);
 
   // Desligado: descarta o áudio e não envia nada
   if (!transmitindo || n == 0) return;
 
   for (int i = 0; i < n; i++) {
     int32_t s = amostras[i] >> 8;            // 24 bits
     dc = dc * 0.999f + s * 0.001f;           // remove offset DC
     float x = (s - dc) / 256.0f * GANHO;     // 24 -> 16 bits, com ganho fixo
     if (x > 32767) x = 32767;
     if (x < -32768) x = -32768;
     int16_t v = (int16_t)x;
     estereo[i * 2]     = v;                  // canal esquerdo
     estereo[i * 2 + 1] = v;                  // canal direito
   }
 
   a2dp.write((uint8_t*)estereo, n * 2 * sizeof(int16_t));
 }