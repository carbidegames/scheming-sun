# Data Flow
Player input data flows through the engine in the following path:
- Client receives W key-press data from the OS
- Client translates W into a "Move Forward" button change event
- Client sends the event to the back-end
- Back-end receives the event and switches the "Move Forward" state to true
- As the back-end sends a frame of data to the server, it includes the
    "Move Forward" state
- As the back-end executes a world tick, it creates a player prediction state
    for the "Move Forward" state during that current frame
- The back-end receives authoritative data from the server and adjust the player
    position based on what the difference of the player position was at the time
    the server data was accurate
