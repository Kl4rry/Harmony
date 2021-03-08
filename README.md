# Harmony

Harmony is a software soundboard written in Rust  
To inject the audio into your mic to play it in any voice application you need to use something like:  
https://vb-audio.com/Cable/  
https://jackaudio.org/

## Supported Platforms
| OS            | Support|
| ------------- |:------:|
| Windows       | ✅ |
| Linux         | 🚧 |
| MacOS         | 🚧 |

✅ = Tested and working  🚧 = Untested but should work with minimal changes

## Compiling
To build the application use cargo  
A c++ compiler is required as [Harmony](https://github.com/Kl4rry/Harmony) it depends on the [cc crate](https://crates.io/crates/cc)
