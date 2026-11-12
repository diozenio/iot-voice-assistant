#include <BLEDevice.h>
#include <BLEServer.h>
#include <BLEUtils.h>
#include <BLE2902.h>
#include <BluetoothA2DPSource.h>
#include <driver/i2s_std.h>
const char* NOME_FONE = "";
const char* NOME_BLE = "IoT-Voice-ESP32";
#define VOLUME_FONE 100
#define TENTATIVA_FONE_MS 8000
#define PIN_BOTAO 27
#define PIN_LED 23
#define LED_ON LOW
#define LED_OFF HIGH
#define I2S_SCK 32
#define I2S_WS 25
#define I2S_SD 33
#define SAMPLE_RATE 16000
#define GANHO 4.0f
#define HP_CORTE 150.0f

#define UUID_SERVICE "7d0a0001-4a7f-4f6c-9c5e-1f2b3c4d5e6f"
#define UUID_UP "7d0a0002-4a7f-4f6c-9c5e-1f2b3c4d5e6f"
#define UUID_DOWN "7d0a0003-4a7f-4f6c-9c5e-1f2b3c4d5e6f"
#define EV_START 0x01
#define EV_STOP 0x02
#define PLAY_BEGIN 0x01
#define PLAY_END 0x02
#define AUDIO 0x10
#define BLOCK_SAMPLES 256
#define BLOCK_DATA (BLOCK_SAMPLES / 2)
#define PACKET_LEN (1 + 5 + BLOCK_DATA)
#define UP_QUEUE_LEN 64
#define SPK_QUEUE_LEN 160
#define SPK_PREBUFFER 40
typedef struct {
  uint8_t len;
  uint8_t data[PACKET_LEN];
} Packet;

BLECharacteristic* upChar = nullptr;
QueueHandle_t upQueue;
QueueHandle_t spkQueue;
BluetoothA2DPSource a2dp;
volatile bool pcConectado = false;
volatile bool querGravar = false;
volatile bool gravando = false;
volatile bool spkTocando = false;
volatile bool spkIniciou = false;
volatile bool spkFim = false;
static const int8_t INDEX_TABLE[8] = {-1, -1, -1, -1, 2, 4, 6, 8};
static const int16_t STEP_TABLE[89] = {
  7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
  73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408,
  449, 494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066,
  2272, 2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630,
  9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794,
  32767};
struct AdpcmState {
  int32_t predictor = 0;
  int32_t index = 0;
};
static void adpcmApply(AdpcmState& st, uint8_t code) {
  int32_t step = STEP_TABLE[st.index];
  int32_t delta = step >> 3;
  if (code & 4) delta += step;
  if (code & 2) delta += step >> 1;
  if (code & 1) delta += step >> 2;
  st.predictor += (code & 8) ? -delta : delta;
  if (st.predictor > 32767) st.predictor = 32767;
  if (st.predictor < -32768) st.predictor = -32768;
  st.index += INDEX_TABLE[code & 7];
  if (st.index < 0) st.index = 0;
  if (st.index > 88) st.index = 88;
}
static uint8_t adpcmEncode(AdpcmState& st, int16_t sample) {
  int32_t step = STEP_TABLE[st.index];
  int32_t diff = (int32_t)sample - st.predictor;
  uint8_t code = 0;
  if (diff < 0) {
    code = 8;
    diff = -diff;
  }
  if (diff >= step) { code |= 4; diff -= step; }
  step >>= 1;
  if (diff >= step) { code |= 2; diff -= step; }
  step >>= 1;
  if (diff >= step) { code |= 1; }
  adpcmApply(st, code);
  return code;
}
static void montarPacote(Packet& p, AdpcmState& st, uint16_t seq, const int16_t* amostras) {
  p.len = PACKET_LEN;
  p.data[0] = AUDIO;
  p.data[1] = seq & 0xFF;
  p.data[2] = seq >> 8;
  p.data[3] = (uint16_t)st.predictor & 0xFF;
  p.data[4] = (uint16_t)st.predictor >> 8;
  p.data[5] = st.index;
  for (int i = 0; i < BLOCK_DATA; i++) {
    uint8_t lo = adpcmEncode(st, amostras[i * 2]);
    uint8_t hi = adpcmEncode(st, amostras[i * 2 + 1]);
    p.data[6 + i] = lo | (hi << 4);
  }
}
static void decodificarPacote(const uint8_t* d, int16_t* saida) {
  AdpcmState st;
  st.predictor = (int16_t)(d[3] | (d[4] << 8));
  st.index = d[5] > 88 ? 88 : d[5];
  for (int i = 0; i < BLOCK_DATA; i++) {
    adpcmApply(st, d[6 + i] & 0x0F);
    saida[i * 2] = st.predictor;
    adpcmApply(st, d[6 + i] >> 4);
    saida[i * 2 + 1] = st.predictor;
  }
}
float hp_b0, hp_b1, hp_b2, hp_a1, hp_a2;
float hp_x1 = 0, hp_x2 = 0, hp_y1 = 0, hp_y2 = 0;
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
i2s_chan_handle_t rx_chan;
void iniciarMicrofone() {
  i2s_chan_config_t chan_cfg = I2S_CHANNEL_DEFAULT_CONFIG(I2S_NUM_0, I2S_ROLE_MASTER);
  chan_cfg.dma_desc_num = 8;
  chan_cfg.dma_frame_num = 256;
  i2s_new_channel(&chan_cfg, NULL, &rx_chan);
  i2s_std_config_t std_cfg = {
    .clk_cfg = I2S_STD_CLK_DEFAULT_CONFIG(SAMPLE_RATE),
    .slot_cfg = I2S_STD_PHILIPS_SLOT_DEFAULT_CONFIG(I2S_DATA_BIT_WIDTH_32BIT, I2S_SLOT_MODE_MONO),
    .gpio_cfg = {
      .mclk = I2S_GPIO_UNUSED,
      .bclk = (gpio_num_t)I2S_SCK,
      .ws = (gpio_num_t)I2S_WS,
      .dout = I2S_GPIO_UNUSED,
      .din = (gpio_num_t)I2S_SD,
      .invert_flags = {false, false, false},
    },
  };
  std_cfg.slot_cfg.slot_mask = I2S_STD_SLOT_LEFT;
  i2s_channel_init_std_mode(rx_chan, &std_cfg);
  i2s_channel_enable(rx_chan);
}
static void enfileirarEvento(uint8_t evento) {
  Packet p;
  p.len = 1;
  p.data[0] = evento;
  xQueueSend(upQueue, &p, 0);
}

void tarefaMicrofone(void*) {
  static int32_t bruto[BLOCK_SAMPLES];
  static int16_t bloco[BLOCK_SAMPLES];
  AdpcmState codec;
  uint16_t seq = 0;
  int preenchido = 0;
  for (;;) {
    size_t lidos = 0;
    i2s_channel_read(rx_chan, bruto, sizeof(bruto), &lidos, 100);
    int n = lidos / sizeof(int32_t);
    if (querGravar && !gravando) {
      gravando = true;
      zerarFiltro();
      codec = AdpcmState();
      seq = 0;
      preenchido = 0;
      enfileirarEvento(EV_START);
    } else if (!querGravar && gravando) {
      if (preenchido > 0) {
        memset(bloco + preenchido, 0, (BLOCK_SAMPLES - preenchido) * sizeof(int16_t));
        Packet p;
        montarPacote(p, codec, seq++, bloco);
        xQueueSend(upQueue, &p, 0);
        preenchido = 0;
      }
      gravando = false;
      enfileirarEvento(EV_STOP);
    }

    if (!gravando) continue;
    for (int i = 0; i < n; i++) {
      float x = filtrarHP((bruto[i] >> 8) / 256.0f) * GANHO;
      if (x > 32767) x = 32767;
      if (x < -32768) x = -32768;
      bloco[preenchido++] = (int16_t)x;
      if (preenchido == BLOCK_SAMPLES) {
        Packet p;
        montarPacote(p, codec, seq++, bloco);
        if (xQueueSend(upQueue, &p, 0) != pdTRUE) {
          Serial.println("Fila BLE cheia: pacote de áudio descartado");
        }
        preenchido = 0;
      }
    }
  }
}
void tarefaEnvio(void*) {
  Packet p;
  for (;;) {
    xQueueReceive(upQueue, &p, portMAX_DELAY);
    if (!pcConectado) continue;
    upChar->setValue(p.data, p.len);
    upChar->notify();
    vTaskDelay(pdMS_TO_TICKS(2));
  }
}

int32_t dadosFone(Frame* frames, int32_t quantidade) {
  static int16_t bloco[BLOCK_SAMPLES];
  static int posicao = BLOCK_SAMPLES;
  static int16_t anterior = 0, atual = 0;
  static float fase = 0;
  const float passo = (float)SAMPLE_RATE / 44100.0f;
  if (!spkTocando) return 0;
  if (!spkIniciou) {
    if (uxQueueMessagesWaiting(spkQueue) >= SPK_PREBUFFER || spkFim) {
      spkIniciou = true;
      posicao = BLOCK_SAMPLES;
      anterior = atual = 0;
      fase = 0;
    } else {
      for (int i = 0; i < quantidade; i++) frames[i].channel1 = frames[i].channel2 = 0;
      return quantidade;
    }
  }
  for (int i = 0; i < quantidade; i++) {
    fase += passo;
    while (fase >= 1.0f) {
      if (posicao >= BLOCK_SAMPLES) {
        uint8_t pacote[PACKET_LEN];
        if (xQueueReceive(spkQueue, pacote, 0) == pdTRUE) {
          decodificarPacote(pacote, bloco);
          posicao = 0;
        } else if (spkFim) {
          spkTocando = false;
          return i;
        } else {
          anterior = atual;
          atual = 0;
          fase -= 1.0f;
          continue;
        }
      }
      anterior = atual;
      atual = bloco[posicao++];
      fase -= 1.0f;
    }
    int16_t v = anterior + (int16_t)((atual - anterior) * fase);
    frames[i].channel1 = v;
    frames[i].channel2 = v;
  }
  return quantidade;
}
class ServidorCallbacks : public BLEServerCallbacks {
  void onConnect(BLEServer* server, esp_ble_gatts_cb_param_t* param) override {
    pcConectado = true;
    server->updateConnParams(param->connect.remote_bda, 6, 12, 0, 400);
    Serial.println("Computador conectado (BLE)");
  }
  void onDisconnect(BLEServer* server) override {
    pcConectado = false;
    querGravar = false;
    spkTocando = false;
    Serial.println("Computador desconectado, anunciando de novo");
    BLEDevice::startAdvertising();
  }
};
class DownCallbacks : public BLECharacteristicCallbacks {
  void onWrite(BLECharacteristic* c) override {
    const uint8_t* d = c->getData();
    size_t len = c->getLength();
    if (len == 0) return;
    switch (d[0]) {
      case PLAY_BEGIN:
        xQueueReset(spkQueue);
        spkFim = false;
        spkIniciou = false;
        spkTocando = true;
        break;
      case PLAY_END:
        spkFim = true;
        break;
      case AUDIO:
        if (len == PACKET_LEN && spkTocando) xQueueSend(spkQueue, d, 0);
        break;
    }
  }
};
void iniciarBLE() {
  BLEDevice::init(NOME_BLE);
  BLEDevice::setMTU(247);
  BLEServer* server = BLEDevice::createServer();
  server->setCallbacks(new ServidorCallbacks());
  BLEService* service = server->createService(UUID_SERVICE);
  upChar = service->createCharacteristic(UUID_UP, BLECharacteristic::PROPERTY_NOTIFY);
  upChar->addDescriptor(new BLE2902());
  BLECharacteristic* down = service->createCharacteristic(
    UUID_DOWN, BLECharacteristic::PROPERTY_WRITE_NR | BLECharacteristic::PROPERTY_WRITE);
  down->setCallbacks(new DownCallbacks());
  service->start();
  BLEAdvertising* adv = BLEDevice::getAdvertising();
  adv->addServiceUUID(UUID_SERVICE);
  adv->setScanResponse(true);
  BLEDevice::startAdvertising();
}

bool botaoAnterior = HIGH;
unsigned long ultimoToque = 0;
const unsigned long DEBOUNCE_MS = 50;

void verificarBotao() {
  bool leitura = digitalRead(PIN_BOTAO);
  if (botaoAnterior == HIGH && leitura == LOW && millis() - ultimoToque > DEBOUNCE_MS) {
    ultimoToque = millis();

    if (!querGravar) {
      if (!pcConectado) {
        Serial.println("Sem computador conectado, ignorando.");
      } else {
        querGravar = true;
        Serial.println(">>> Gravando");
      }
    } else {
      querGravar = false;
      Serial.println(">>> Parado, enviando para transcrição");
    }
  }
  botaoAnterior = leitura;
}

void estadoFone(esp_a2d_connection_state_t estado, void*) {
  switch (estado) {
    case ESP_A2D_CONNECTION_STATE_CONNECTING:
      Serial.println("Fone: conectando...");
      break;
    case ESP_A2D_CONNECTION_STATE_CONNECTED:
      Serial.println("Fone: conectado");
      break;
    case ESP_A2D_CONNECTION_STATE_DISCONNECTING:
      Serial.println("Fone: desconectando...");
      break;
    default:
      Serial.println("Fone: desconectado");
      break;
  }
}

void manterFone() {
  static unsigned long ultimaTentativa = 0;
  if (a2dp.get_connection_state() != ESP_A2D_CONNECTION_STATE_DISCONNECTED) return;
  if (millis() - ultimaTentativa < TENTATIVA_FONE_MS) return;

  ultimaTentativa = millis();
  Serial.printf("Fone '%s' ainda nao conectado, tentando de novo...\n",
                strlen(NOME_FONE) ? NOME_FONE : "(qualquer)");
  a2dp.reconnect();
}

void atualizarLed() {
  bool aceso;
  if (gravando) {
    aceso = true;
  } else if (!pcConectado) {
    aceso = (millis() / 500) % 2 == 0;
  } else {
    aceso = false;
  }
  digitalWrite(PIN_LED, aceso ? LED_ON : LED_OFF);
}
void setup() {
  Serial.begin(115200);
  pinMode(PIN_BOTAO, INPUT_PULLUP);
  pinMode(PIN_LED, OUTPUT);
  digitalWrite(PIN_LED, LED_OFF);
  upQueue = xQueueCreate(UP_QUEUE_LEN, sizeof(Packet));
  spkQueue = xQueueCreate(SPK_QUEUE_LEN, PACKET_LEN);
  iniciarFiltro();
  iniciarMicrofone();

  a2dp.set_default_bt_mode(ESP_BT_MODE_BTDM);
  a2dp.set_auto_reconnect(true);
  a2dp.set_volume(VOLUME_FONE);
  a2dp.set_on_connection_state_changed(estadoFone);
  a2dp.start(NOME_FONE, dadosFone);
  Serial.printf("Procurando o fone '%s'...\n", NOME_FONE);
  iniciarBLE();
  Serial.printf("Anunciando como '%s'\n", NOME_BLE);
  xTaskCreatePinnedToCore(tarefaMicrofone, "mic", 6144, NULL, 5, NULL, 1);
  xTaskCreatePinnedToCore(tarefaEnvio, "ble-tx", 4096, NULL, 4, NULL, 1);
}

void loop() {
  verificarBotao();
  manterFone();
  atualizarLed();
  delay(5);
}