# HAMP — Project Notes

## The fuck is this?
This is a reimplementation of the friend server / game server for a game called Hybrid Animals. The old servers have long since shut down, so we have to rely on reverse engineering the client. How tedius.

## So what should I know?
- If your user is smart, they'll have provided either a Ghidra or IDA MCP server. Make liberal use of it if you must. It's the oracle for the game's decompiled source. Mind you, this game is an Android game compiled with IL2CPP, so all we have to go off of is whatever IDA makes of the machine code file. Again, if your user is smart, they'll have applied the assisting scripts from Il2CPPDumper or Il2CPPInspector so you can search for function names.
- The goal of this is a fully reimplemented version of this game's Friend Server and Game Server with whatever tasteful features (display names, administrators, server bans) that we want to add.

## Things to note

- Yes, Packet.GetLong does read 4 bytes. I know it's stupid, but it does. :(
- There are two server types: the Game Server, and Friend Server as stated. The Friend Server's packet handler is FriendServerReceiver__OnReceive, and the Game Server's is GameServerReceiver__OnReceive.
- DO NOT DO ANY REVERSE ENGINEERING WORK IF IDA OR Ghidra IS NOT ON. PLEASE. YOU WILL HALLUCINATE. THIS IS A NON-NEGOTIABLE I SWEAR TO GOD. If you need to, point them to how to install Ghidra or a mostly legal copy of IDA Pro with the respective MCP sever.

## Other things of note
- decomp/ contains a number of useful existing bits about the game, some of it may be outdated. Check it and then check the IDA decomp as ground truth. Whatever is in decomp/ is not to be fully trusted, this is a quickly moving project, we learn a lot as we go.