# Architecture
The library `sc-client` contains the frontend rendering and main state machine.
It will handle scene switches, such as moving from the loading scene to the main
menu, and from the main menu to the game. It relies on `sc-client-game` to
handle keeping track of game data, connecting to the server, and to perform lag
compensation.
