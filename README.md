# Temp VoiceRS bot

Requirements:
- [ ] Can generate temporary voice channels
	- [ ] Private VCs 
		- [ ] By User ping
		- [ ] By Role ping
	- [ ] Public VCs
		- [ ] Time out overrides by mods in create command?
- [ ] Mod roles can view all channels
	- [ ] Mod override by User ping 
	- [ ] Mod roles customized by Role ping
- [ ] Completely ephemeral 
	- [ ] Delete chat once all users exit VC
		- [ ] Configurable Timeout
	- [ ] Delete Voice chat once all users exit VC
		- [ ] Configurable TImeout
	- [ ] Delete any Bot side logging?

![Flowchart](Images/Flowchart.jpg)

# Crates used
[Serenity](https://crates.io/crates/serenity)
[tracing](https://crates.io/crates/tracing)
[Chrono](https://crates.io/crates/chrono)
[Tokio](https://crates.io/crates/tokio)
