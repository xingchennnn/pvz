/** 播放音乐 */
export class MusicPlayer {
    // 上一次播放时间
    lastPlayTime: number = 0;
    // 当前播放时间
    currentPlayTime: number = 0;
    // 是否正在播放
    isPlaying: boolean = false;
    // 音量
    volume: number = 1;
    // 音乐总时长
    duration: number = 0;
    // 音乐播放器
    url: string = "";
    // 音乐播放器
    audio: HTMLAudioElement | null = null;

    constructor(url: string) {
        this.url = url;
        this.audio = new Audio(url);
    }

    /** 设置音量 */
    setVolume(volume: number) {
        // console.log("设置音量", volume);
        this.volume = volume;
        if (this.audio) {
            this.audio.volume = volume;
        }
    }
    /** 设置播放位置 */
    setCurrentTime(currentTime: number) {
        // console.log("设置播放位置", currentTime);
        this.currentPlayTime = currentTime;
        if (this.audio) {
            this.audio.currentTime = currentTime;
        }
    }

    /** 播放音乐 */
    play() {
        // 调用原生代码播放音乐
        if (this.audio) {
            this.audio.play();
        }
    }

    /** 暂停音乐播放 */
    pause() {
        // 调用原生代码暂停音乐
        if (this.audio) {
            this.audio.pause();
        }
    }
    /** 停止音乐播放 */
    stop() {
        // 调用原生代码停止音乐
        if (this.audio) {
            this.audio.pause();
            this.audio.currentTime = 0;
        }
    }
    /** 退出音乐播放器 */
    quit() {
        // 销毁音乐播放器
        if (this.audio) {
            this.audio.pause();
            this.audio.src = "";
            this.audio.load();
            this.audio = null;
        }
    }
}