# hawkeye-pi

Aplicativo Windows nativo para câmeras IP Yoosee / Xiongmai / Anyka. Ver, ouvir e girar a câmera sem depender do app oficial nem da nuvem chinesa.

<p align="center">
  <img src="ui/icon-512.png" width="128" alt="hawkeye-pi">
</p>

<p align="center">
  <a href="https://github.com/joaosouz4dev/hawkeye-pi/releases/latest">
    <b>⬇️ Baixar a última versão</b>
  </a>
</p>

## O que é

Um app **nativo do Windows** que abre uma janela mostrando o vídeo ao vivo da sua câmera IP com:

- Streaming Full HD (H.264 transcodado do H.265 nativo)
- Controle PTZ (mover a câmera em 8 direções)
- Snapshot, gravação de clipe (30s), tela cheia, replay instantâneo dos últimos ~30s
- Auto-hide dos controles estilo Netflix
- Áudio bidirecional? **Não, veja "O que não tem" abaixo**

Tudo em uma janela nativa. Sem terminal, sem servidor no navegador, sem depender de nuvem. Fecha a janela → mata tudo.

## Instalação

**Rápido:** baixe o instalador `.exe` ou `.msi` da [última Release](https://github.com/joaosouz4dev/hawkeye-pi/releases/latest) e execute.

**Primeira execução:** o app abre uma tela pedindo IP e senha da câmera. Preencha uma vez.

**Configuração:** salvos em `%LOCALAPPDATA%\hawkeye-pi\config.local.yaml`. Para trocar de câmera ou senha, edite esse arquivo e reabra o app.

### ⚠️ Antivírus

O `.exe` **não é assinado digitalmente** (certificado code-signing custa alguns centavos por ano e este projeto é gratuito). Alguns antivírus, especialmente **Kaspersky** e configurações agressivas de outros, podem quarentenar o executável ao rodar pela primeira vez. **Windows Defender padrão não bloqueia.**

Se seu antivírus reclamar:
- Verifique o SHA-256 do arquivo em [Releases](https://github.com/joaosouz4dev/hawkeye-pi/releases/latest)
- Compare com o hash publicado no changelog da versão
- Adicione uma exceção no seu antivírus para `%LOCALAPPDATA%\Programs\hawkeye-pi\`

O código-fonte inteiro está aqui — você pode auditar exatamente o que o app faz.

### Pré-requisitos da câmera

- Câmera Yoosee/Xiongmai/Anyka (SoC AK3918 — várias marcas white-label)
- ONVIF ativado no app oficial: `Configurações` → `Conexão NVR` → `Ativar a conexão`
- Câmera na mesma rede local que o PC

## Como funciona por dentro

```
┌──────────────────────────────────────────┐
│           Janela do App (WebView2)       │
│  ┌────────────────────────────────────┐  │
│  │  UI (HTML/CSS/JS) - a interface    │  │
│  │  que voce ja conhece               │  │
│  └────────────────────────────────────┘  │
│           │                              │
│           │ localhost:PORTA_ALEATORIA    │
│           ▼                              │
│  ┌────────────────────────────────────┐  │
│  │  Servidor HTTP local (Rust/axum)   │  │
│  │  - /api/config, /api/setup         │  │
│  │  - /move/<dir>, /stop  (ONVIF)     │  │
│  │  - serve /ui/* estatico            │  │
│  └────────────────────────────────────┘  │
│           │                              │
│           │ spawn (invisivel)            │
│           ▼                              │
│  ┌────────────────────────────────────┐  │
│  │  go2rtc.exe (sidecar embutido)     │  │
│  │  RTSP -> MSE/WebRTC                │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
           │              │
           │ ONVIF        │ RTSP
           ▼              ▼
        ┌────────────────────┐
        │   Câmera IP        │
        └────────────────────┘
```

- **App Tauri (Rust):** janela nativa Windows via WebView2 (Edge). Sem terminal, ícone próprio.
- **Servidor local:** loopback em porta aleatória (invisível pra fora). Ninguém na rede consegue conectar.
- **go2rtc:** empacotado dentro do `.exe`. Roda como sidecar invisível, morre quando o app fecha.
- **Config:** `%LOCALAPPDATA%\hawkeye-pi\config.local.yaml`

## O que NÃO tem (e por quê)

Testei exaustivamente, essas features não funcionam por limitação do **firmware** da câmera:

- **Presets de posição** — `SetPreset`/`GotoPreset`/`AbsoluteMove`/`GetStatus` do ONVIF respondem `0 bytes`. Sem posição absoluta, presets seriam frágeis. Use presets pelo app oficial quando precisar.
- **Microfone (falar pela câmera)** — RTSP `ANNOUNCE` retorna `405 Method Not Allowed` e ONVIF audio backchannel responde `0 bytes`. O firmware desabilita upload de áudio deliberadamente.
- **Zoom óptico** — câmera "360" tem lente fixa; o "360" é o giro pan, não zoom.

Essas ausências são limitações reais da câmera, não do projeto. Confirmadas por 5 projetos de referência (PTZ-YALL, PTZ-YCC365, yooseeptz, camera-hack, onvif-go).

## Desenvolvimento

Precisa de:
- **Rust stable** (via [rustup](https://rustup.rs))
- **MSVC Build Tools** (link.exe) — do Visual Studio Build Tools
- **tauri-cli**: `cargo install tauri-cli --locked`
- **go2rtc.exe** em `src-tauri/binaries/go2rtc-x86_64-pc-windows-msvc.exe` (já vem no repo)

```powershell
# clone
git clone https://github.com/joaosouz4dev/hawkeye-pi.git
cd hawkeye-pi

# rodar em modo dev (compila + abre janela com hot-reload)
cd src-tauri
cargo tauri dev

# compilar release (gera .msi e .exe em src-tauri/target/release/bundle/)
cargo tauri build
```

### Release automático

`git tag v0.X.Y && git push --tags` dispara um workflow do GitHub Actions que compila em windows-latest e publica o `.msi` + `.exe` como Release.

## Segurança

- O app não abre portas na rede: o servidor HTTP interno escuta **só em `127.0.0.1`** com porta aleatória.
- Config e credenciais ficam em `%LOCALAPPDATA%\hawkeye-pi\` (só o seu usuário Windows lê).
- **Não** exponha a câmera diretamente na internet — se precisar acesso remoto, use VPN (Tailscale, WireGuard).

## Créditos

- [go2rtc](https://github.com/AlexxIT/go2rtc) por AlexxIT — o coração do streaming.
- [Tauri](https://tauri.app) — framework nativo Windows/Linux/Mac em Rust.
- [PTZ-YALL](https://github.com/carvalr/PTZ-YALL), [yooseeptz](https://github.com/RICARDOKR/yooseeptz), [camera-hack](https://github.com/gabrielmaialva33/camera-hack) — engenharia reversa do firmware Yoosee/Anyka.

## Licença

MIT. Ver [LICENSE](LICENSE).
