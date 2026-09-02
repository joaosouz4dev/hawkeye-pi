# hawkeye-pi

Interface web imersiva e leve para câmeras IP Yoosee / Xiongmai / Anyka, controle PTZ e streaming HD sem depender do app oficial nem da nuvem.

![banner](icon-512.png)

Faz o que o app do fabricante faz de bom (ver, ouvir, girar) e mostra tudo no navegador do PC ou do celular. Instala como app (PWA). Roda 100% na sua rede local — o vídeo não passa pelos servidores do fabricante.

## Por que existe

Câmeras Yoosee (e as ~15 marcas white-label baseadas no SoC Anyka AK3918: Jortan, Kapbom, Basike, Dura Well, GNCC, TECKIN, LETSCEE, Hiseeu, ANRAN, Ctronics, GWIPC e outras) implementam ONVIF de forma **parcial**: só `ContinuousMove` e `Stop` respondem. `AbsoluteMove`, `SetPreset`, `GotoPreset`, `GetStatus`, backchannel de áudio — todos ignorados pelo firmware. E o app do fabricante conversa com a câmera por um protocolo P2P proprietário cifrado que roteia pela nuvem chinesa.

Este projeto pega o que **funciona** do ONVIF (movimento) e junta com o [go2rtc](https://github.com/AlexxIT/go2rtc) (que bridge RTSP → WebRTC/MSE) para entregar uma UI web moderna, offline-first, sem nuvem.

## Features

- **Streaming Full HD** ao vivo via MSE (H.264 transcodado do H.265 nativo)
- **Auto-hide dos overlays** após 3s parados (estilo Netflix), reaparece ao mover mouse/tocar
- **Controle PTZ** por d-pad de 8 direções (mouse, toque e teclas de seta)
- **Snapshot** — baixa foto JPEG instantânea
- **Rec-clipe** — grava 30 segundos como `.webm` no seu PC
- **Fullscreen** nativo do navegador
- **Instant replay** — barra que scrubs os últimos ~30s bufferizados
- **Estado real de conexão** — badge "ao vivo" / "reconectando" / "replay" baseado em fluxo real de frames (não em WebSocket)
- **PWA instalável** — vira app no PC (Chrome/Edge) ou home do celular
- **Layout responsivo** — mobile portrait, mobile landscape e desktop
- **Áudio** com botão de mudo (inicia mudo por padrão)
- **Atalhos de teclado** — setas movem, `S` snapshot, `R` rec, `F` fullscreen, `M` mudo

## O que NÃO tem (e por quê)

- **Presets de posição** — Firmware desta família ignora `SetPreset`/`GotoPreset`/`AbsoluteMove`/`GetStatus`. Sem posição absoluta, presets só podem ser aproximados por tempo (frágeis) ou por visão computacional (frágeis com chuva/pouca luz). Testados exaustivamente, removidos por não serem confiáveis. Use presets pelo app oficial quando precisar.
- **Microfone (falar pela câmera)** — Firmware retorna `405 Method Not Allowed` para RTSP `ANNOUNCE` e `0 bytes` para todos os métodos ONVIF de áudio bidirecional. Bloqueado deliberadamente. Só o app P2P do fabricante consegue.
- **Zoom** — a câmera 360 tem lente fixa, o "360" é só o giro pan (não zoom óptico). `GetNodes` do ONVIF confirma: nenhum eixo de zoom.

Essas ausências são limitações da câmera, não do projeto — verifique você mesmo antes de reclamar 😉

## Arquitetura

```
┌─────────────┐   RTSP    ┌──────────┐   MSE/WebRTC   ┌──────────┐
│ Câmera IP   │──────────▶│ go2rtc   │───────────────▶│ Navegador│
│ (Yoosee)    │  UDP:554  │ :1984    │  ws (transcode)│  (PWA)   │
└─────────────┘           └──────────┘                └──────────┘
      ▲                                                     │
      │ ONVIF ContinuousMove/Stop (TCP:5000)                │
      │                    ┌──────────────┐  HTTP POST      │
      └────────────────────│ptz_control.py│◀────────────────┘
                           │ :1985 (Python│
                           │  stdlib puro)│
                           └──────────────┘
```

- **go2rtc** faz o trabalho pesado: bufferiza o RTSP, transcodifica H.265 → H.264 quando o navegador precisa (MSE), oferece WebRTC quando possível.
- **ptz_control.py** é um servidor HTTP mínimo (só stdlib do Python) que:
  1. Serve a UI (HTML/CSS/JS embutidos)
  2. Recebe comandos `POST /move/<dir>` e `POST /stop` da UI
  3. Traduz para SOAP ONVIF com WS-Security (UsernameToken PasswordDigest SHA-1) e envia para a câmera
- **PWA** — manifest + service worker mínimo. Sem cache offline (não faz sentido pra streaming ao vivo).

## Pré-requisitos

- **Windows / Linux / Mac** com Python 3.10+
- **[go2rtc](https://github.com/AlexxIT/go2rtc/releases)** (binário standalone, 19MB)
- **ffmpeg** no PATH (para transcode H.265 → H.264)
- Câmera Yoosee/Xiongmai/Anyka acessível na rede local, com ONVIF ativado no app (`Configurações` → `Conexão NVR` → `Ativar a conexão`)

## Instalação

```bash
git clone https://github.com/joaosouz4dev/hawkeye-pi.git
cd hawkeye-pi
```

**1. go2rtc** — baixe o binário e coloque na pasta do projeto. Copie `go2rtc.example.yaml` como `go2rtc.yaml` e edite as URLs RTSP com o IP e senha da sua câmera.

**2. ptz_control** — copie `config.example.yaml` como `config.local.yaml` e preencha com os dados da câmera:

```yaml
cam_host: 192.168.1.100
cam_user: admin
cam_pass: sua_senha_onvif
cam_label: Garagem
```

**3. Rodar**

```bash
./go2rtc.exe -c go2rtc.yaml
python ptz_control.py
```

**4. Abrir** `http://localhost:1985` no navegador. Pronto.

## Instalar como app (PWA)

**Desktop (Chrome/Edge):** com a página aberta, clique no ícone de instalar na barra de endereço (ou menu → "Instalar hawkeye-pi"). Vira um ícone no menu iniciar/dock.

**Celular (Android/iOS):** menu do navegador → "Adicionar à tela inicial". Vira ícone na home, abre em tela cheia sem barra do navegador.

## Iniciar com o sistema (Windows)

Cria uma tarefa agendada que sobe no logon:

```powershell
$py = "C:\Users\SEU_USUARIO\AppData\Local\Programs\Python\Python312\pythonw.exe"
$action = New-ScheduledTaskAction -Execute $py -Argument "C:\hawkeye-pi\ptz_control.py" -WorkingDirectory "C:\hawkeye-pi"
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
Register-ScheduledTask -TaskName "hawkeye-pi" -Action $action -Trigger $trigger -Force
```

Faça o mesmo para o go2rtc apontando para o `go2rtc.exe`.

## Descobrindo a URL RTSP da sua câmera

Se você não sabe o caminho RTSP (varia por firmware), teste em ordem:

- Yoosee/Xiongmai (Anyka): `rtsp://user:senha@IP:554/onvif1` (main) e `/onvif2` (sub) via **UDP**
- Genérico H.264: `rtsp://user:senha@IP:554/stream1` ou `/h264` ou `/Streaming/Channels/101`
- ffprobe para testar: `ffprobe -rtsp_transport udp rtsp://user:senha@IP:554/onvif1`

## Segurança

Este projeto expõe a interface **apenas em `localhost` (127.0.0.1)** por padrão. Se você quiser acesso remoto, **não** faça port-forward direto no roteador — use uma VPN (Tailscale, WireGuard) ou um reverse proxy com autenticação e HTTPS. A UI não tem login: quem alcançar a porta 1985 controla a câmera.

## Créditos e referências

- [go2rtc](https://github.com/AlexxIT/go2rtc) por AlexxIT — o coração do streaming
- [PTZ-YALL](https://github.com/carvalr/PTZ-YALL) e [yooseeptz](https://github.com/RICARDOKR/yooseeptz) — inspiração e confirmação das quirks do firmware Yoosee
- [camera-hack](https://github.com/gabrielmaialva33/camera-hack) — engenharia reversa detalhada do SoC Anyka AK3918

## Licença

MIT. Faz o que quiser, sem garantia.
