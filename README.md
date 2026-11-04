# IoT Voice Assistant

Aplicativo desktop (Tauri + React) que roda o servidor WebSocket do assistente de voz
nativamente, dentro do próprio programa. Tudo que antes era impresso no terminal do
servidor Node aparece no painel do aplicativo: status do servidor, dispositivos
conectados, transcrições, respostas da IA, gravações e o console de log.

## Como funciona

O servidor WebSocket sobe automaticamente quando o aplicativo abre e fica ouvindo em
`ws://<ip-da-rede-local>:3000`. O fluxo por conexão é o mesmo do servidor original:

1. O ESP32 envia o texto `start` e começa a transmitir PCM 16 bits mono a 16 kHz em
   frames binários.
2. Ao receber `stop`, o áudio acumulado é convertido para WAV e salvo em disco.
3. O WAV é enviado para o endpoint `/audio/transcriptions` da OpenRouter e transcrito
   por um modelo dedicado de transcrição (padrão `openai/gpt-4o-transcribe`).
4. A transcrição entra no histórico da conversa e é respondida pelo modelo de IA.
5. A resposta volta para o ESP32 pelo mesmo WebSocket.

Gravações com menos de 0,5 s são descartadas, e um `start` que chegue enquanto o comando
anterior ainda está sendo processado é ignorado — como no original.

## Painel

- **Monitor** — status e endereço do servidor, conexões ativas com estado
  (ocioso / gravando / processando), conversa (transcrição + resposta) e console de log.
- **Gravações** — lista dos WAV da sessão, com atalho para abrir o arquivo ou a pasta.
- **Configurações** — porta, chave da OpenRouter e modelos de transcrição e de resposta.

Ao abrir, o painel aparece centralizado e com foco. Fechar a janela apenas a esconde —
o programa continua rodando, e tanto o ícone da bandeja quanto o ícone no Dock reabrem
o painel. Encerrar de vez é pela opção **Fechar** do menu da bandeja.

## Configuração

Os valores padrão vêm do `.env` na raiz do projeto, embutidos no binário em tempo de
compilação (por isso o executável é portável — não precisa de `.env` ao lado dele):

```
PORT=3000
OPENROUTER_API_KEY=
MODELO_TRANSCRICAO=openai/gpt-4o-transcribe
MODELO_IA=google/gemini-2.5-flash
```

`MODELO_TRANSCRICAO` precisa ser um modelo de transcrição — `openai/gpt-4o-transcribe`,
`openai/gpt-4o-mini-transcribe` ou `openai/whisper-1`. Modelos de chat com entrada de
áudio não servem aqui: eles tendem a *responder* ao áudio em vez de transcrevê-lo.

O que for salvo na aba **Configurações** vai para `settings.json` na pasta de
configuração do aplicativo e tem prioridade sobre esses padrões. Mudanças de porta
exigem reiniciar o servidor (há o botão **Salvar e reiniciar**).

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

Os sketches do ESP32 estão em `firmware/` (`esp.ino` e `fone.ino`). Ajuste `SERVER_IP`
e `SERVER_PORT` para o endereço mostrado no painel.
