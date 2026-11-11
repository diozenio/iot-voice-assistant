# IoT Voice Assistant

Aplicativo desktop (Tauri + React) que conversa com um ESP32 por **Bluetooth**. O ESP32
captura a voz, o aplicativo transcreve, a IA decide a ação (ou responde) e a resposta é
falada no fone de ouvido que está conectado ao próprio ESP32. Status, conversa,
gravações e o console de log aparecem no painel.

## Como funciona

O ESP32 fica conectado a dois dispositivos ao mesmo tempo: o computador (BLE) e o fone
(A2DP). Quem conecta ao ESP32 é o computador — ele aparece na lista **Bluetooth** do painel
e você clica em **Conectar**.

1. Aperte o botão do ESP32: ele envia `start` e transmite o microfone (16 kHz mono,
   comprimido em ADPCM 4:1) por BLE. Apertar de novo envia `stop`.
2. O áudio é decodificado, salvo em WAV, limpo com supressão de ruído e enviado ao endpoint
   `/audio/transcriptions` da OpenRouter (padrão `openai/gpt-4o-transcribe`, em português).
3. A transcrição entra no histórico da conversa e o modelo de IA escolhe uma das ações
   predefinidas (veja abaixo) ou **responder** (qualquer outro pedido).
4. O aplicativo executa a ação, converte a resposta em voz pelo endpoint `/audio/speech` da
   OpenRouter e a envia ao ESP32 por BLE, que a toca no fone via A2DP.

Gravações com menos de 0,5 s são descartadas, e um `start` que chegue enquanto o comando
anterior ainda está sendo processado é ignorado.

### Por que não corta

- O ESP32 separa as tarefas por núcleo: a pilha Bluetooth fica no núcleo 0 e o microfone,
  o ADPCM e o envio BLE ficam no núcleo 1, ligados por filas (~1 s de folga).
- O ADPCM reduz o áudio para ~64 kbps, bem abaixo do que o BLE sustenta.
- Cada pacote tem número de sequência e é decodificável sozinho. Se algum se perder, o
  aplicativo preenche com silêncio, mantém a duração certa e avisa no log.
- A resposta falada é enviada com um colchão de ~1,3 s e depois no ritmo de reprodução,
  então o buffer do ESP32 não estoura.

## Comandos de voz

| Ação | Exemplo de fala |
| --- | --- |
| Abrir o navegador (sem pesquisa) | "abrir navegador" |
| Pesquisar no Google | "pesquisar receita de bolo" |
| Pesquisar no YouTube | "pesquisar no YouTube música relaxante" |
| Abrir um site | "abrir github.com" |
| Abrir o explorador de arquivos | "abrir explorador" |
| Abrir uma pasta (downloads, documentos, área de trabalho, imagens) | "abrir downloads" |
| Abrir a calculadora | "abrir calculadora" |
| Responder | qualquer outra pergunta |

As ações ficam em `src-tauri/src/actions.rs`; para criar uma nova, adicione a variante em
`Action`, descreva-a no `SYSTEM_PROMPT` e trate-a em `execute`.

## Painel

- **Monitor** — status do Bluetooth, lista de dispositivos encontrados (com sinal, botões
  **Buscar**, **Conectar** e **Desconectar**), conversa (transcrição + resposta) e console de log.
- **Gravações** — lista dos WAV da sessão, com atalho para abrir o arquivo ou a pasta.
- **Configurações** — chave da OpenRouter e modelos de transcrição, de resposta e de voz.

Ao abrir, o painel aparece centralizado e com foco. Fechar a janela apenas a esconde —
o programa continua rodando, e tanto o ícone da bandeja quanto o ícone no Dock reabrem
o painel. Encerrar de vez é pela opção **Fechar** do menu da bandeja.

## Configuração

Os valores padrão vêm do `.env` na raiz do projeto, embutidos no binário em tempo de
compilação (por isso o executável é portável — não precisa de `.env` ao lado dele):

```
OPENROUTER_API_KEY=
MODELO_TRANSCRICAO=openai/gpt-4o-transcribe
MODELO_IA=google/gemini-2.5-flash
MODELO_VOZ=elevenlabs/eleven-flash-v2.5
VOZ=sarah
```

`MODELO_TRANSCRICAO` precisa ser um modelo de transcrição — `openai/gpt-4o-transcribe`,
`openai/gpt-4o-mini-transcribe` ou `openai/whisper-1`. Modelos de chat com entrada de
áudio não servem aqui: eles tendem a *responder* ao áudio em vez de transcrevê-lo.

O que for salvo na aba **Configurações** vai para `settings.json` na pasta de
configuração do aplicativo e tem prioridade sobre esses padrões e valem já na próxima gravação.

As gravações ficam em `gravacoes/` dentro da pasta de dados do aplicativo.

## Desenvolvimento

```bash
pnpm install
pnpm tauri dev
```

Build de distribuição:

```bash
pnpm tauri build
```

## Firmware

O firmware do ESP32 está em `firmware/esp.ino`: microfone → BLE → computador e resposta →
fone (A2DP). Ajuste `NOME_FONE` para o nome do seu fone. Requer a biblioteca **ESP32-A2DP** e
a placa *ESP32 Dev Module* com a partição **Huge APP** (Bluetooth clássico + BLE não cabem
na partição padrão).

No macOS, o primeiro uso pede permissão de Bluetooth (em `pnpm tauri dev`, para o terminal).
