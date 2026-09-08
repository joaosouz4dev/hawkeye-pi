import {VideoRTC} from './video-rtc.js';

/**
 * Player go2rtc customizado que INICIA MUDO.
 * Baseado no video-stream.js oficial, com duas mudancas:
 *  - o <video> nasce muted (nao toca som ao abrir a pagina)
 *  - expoe helpers para ligar/desligar o som a partir da pagina
 */
class VideoStreamMuted extends VideoRTC {
    set divMode(value) {
        const el = this.querySelector('.mode');
        if (el) el.innerText = value;
        const st = this.querySelector('.status');
        if (st) st.innerText = '';
    }

    set divError(value) {
        const el = this.querySelector('.mode');
        if (!el || el.innerText !== 'loading') return;
        el.innerText = 'error';
        this.querySelector('.status').innerText = value;
    }

    oninit() {
        super.oninit();
        // ponto-chave: inicia mudo E sem os controles nativos (temos nossos overlays)
        this.video.muted = true;
        this.video.controls = false;
        this.video.setAttribute('playsinline', '');
        this.video.setAttribute('disablepictureinpicture', '');

        this.innerHTML = `
        <style>
        video-stream { position: relative; }
        .info { position:absolute; top:0; left:0; right:0; padding:10px;
                color:#fff; display:flex; justify-content:space-between;
                pointer-events:none; font-size:.75rem; text-shadow:0 1px 2px #000; }
        </style>
        <div class="info"><div class="status"></div><div class="mode"></div></div>`;
        const info = this.querySelector('.info');
        this.insertBefore(this.video, info);
        // reforca estado apos re-anexar (caso o elemento seja recriado internamente)
        this.video.muted = true;
        this.video.controls = false;
    }

    onconnect() {
        const result = super.onconnect();
        if (result) this.divMode = 'loading';
        return result;
    }

    onopen() {
        const result = super.onopen();
        this.onmessage['stream'] = msg => {
            switch (msg.type) {
                case 'error': this.divError = msg.value; break;
                case 'mse': case 'hls': case 'mp4': case 'mjpeg':
                    this.divMode = msg.type.toUpperCase(); break;
            }
        };
        return result;
    }

    onpcvideo(ev) {
        super.onpcvideo(ev);
        if (this.pcState !== WebSocket.CLOSED) this.divMode = 'RTC';
    }

    // helpers usados pela pagina
    setMuted(m) { if (this.video) { this.video.muted = m; if (!m) this.video.play().catch(()=>{}); } }
    isMuted() { return this.video ? this.video.muted : true; }
}

customElements.define('video-stream', VideoStreamMuted);
