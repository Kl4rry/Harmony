![alt text](https://github.com/Kl4rry/Harmony/blob/main/res/logo.png "Logo")
# Harmony

Harmony is a software soundboard written in Rust  
To inject the audio into your mic to play it in any voice application you need to use something like:  
https://vb-audio.com/Cable/   
https://jackaudio.org/

## Supported Platforms
| OS            | Support|
| ------------- |:------:|
| Windows       | ✅ |
| Linux         | 🆗 |
| MacOS         | 🆗 |

✅ = Tested and working 🆗 = Untested but should work with minimal changes

## Supported Codecs
| OS            | Support|
| ------------- |:------:|
| WAV           | ✅ |
| MP3           | ✅ |
| FLAC          | ✅ |
| Vorbis        | 🆗 |

✅ = Fully supported 🆗 = Playback supported but other features may be incomplete

Vorbis playback is supported but some features like seeing duration of sound are not working yet.

## Install instruction
comming soon...

## Compiling
To build the application use cargo.  
A c++ compiler is required as [Harmony](https://github.com/Kl4rry/Harmony) it depends on the [cc crate](https://crates.io/crates/cc).
