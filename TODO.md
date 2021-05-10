Currently working on:
Clearing up how messages, data, and events are separate for each module
Refactoring message types. Some of the ui::Message stuff needs to become backend::Message stuff
Sending connection info from the server to the UI in order to display it (Figure out how to send the server status through a thread. Perhaps only use ErrorKind?)
Make UI scenes a trait


1. Make program compile
2. Turn UI scenes into trait instead of enum. Perhaps keep the enum as SceneKind if necessary? 
3. Make some tangible progress
   1. Get initial server logic running
      1. Figure out how to let the server run on in its own thread, while still allowing for easy access to the connection info from the application state machine
   
4. Refactor
   1. Make "Scene" an enum, where the variants are structs with the data they need to hold 
      1. Think more about this: https://www.reddit.com/r/rust/comments/buqgam/enum_variants_as_types/
      2. Consider using traits instead?
         - Yeah, use a trait "Draw" for example, so you can directly call scene.draw(terminal) instead of having to match against the different possible scenes
   2. Make the state of the logic state machine an enum instead of a struct, so the UI can use this same enum 

5. Implement stylesheet
   1. Define color constants in a file for this purpose

https://github.com/yasammez/nachricht 


- Send info to UI when server refreshes connection
- Make sure that different types of messages are separated nicely
  - ui::Message should only define stuff that the ui can receive
  - server::Message should only define stuff that the Server can receive
  - If the server wants to send stuff to the ui, it will send a ui::Message