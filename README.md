<p align="center">
  <img src="https://github.com/Kl4rry/Harmony/blob/main/res/logo.png" />
</p>

<p align="center">A software soundboard written in Rust</p>

## Supported Platforms
| OS            | Support|
| ------------- |:------:|
| Windows       | ✅ |
| Linux         | 🆗 |
| MacOS         | 🆗 |

✅ = Tested and working 🆗 = Untested but should work with minimal changes

## Supported Codecs
| Codec         | Playback | Duration |
| ------------- |:------:|:------:|
| WAV           | ✅ | ✅ |
| MP3           | ✅ | ✅ |
| FLAC          | ✅ | ✅ |
| Vorbis        | ✅ | ❌ |

✅ = Supported ❌ = Not supported  
More codecs are going to be supported through ffmpeg conversion.

## 1.0 Roadmap
| Feature         | Completed |
| --------------- |:------:|
| Play hotkey support    | ❌ |
| Pause hotkey support   | ❌ |
| Stop hotkey support    | ❌ |
| youtube-dl integration | ❌ |
| ffmpeg auto conversion | ❌ |
| Seeking in clip        | ❌ |

## Mic injection
To inject the audio into your mic to play it in any voice application you need to use something like:  
https://vb-audio.com/Cable/   
https://jackaudio.org/

## Install instruction
comming soon...

## Building
To build the application use cargo.  
A c++ compiler is required as [Harmony](https://github.com/Kl4rry/Harmony) it depends on the [cc crate](https://crates.io/crates/cc).
