Currently working on:
Turn "Scene" into a trait. Perhaps store all scenes, so their states can be preserved, instead of building new scenes, every time they are switched between? This way, the UI can store the connection info instead of having to retrieve it all the time. 
Sending connection info from the server to the UI in order to display it (Figure out how to send the server status through a thread. Perhaps only use ErrorKind?)
Make UI scenes a trait


1. Turn UI scenes into trait instead of enum. Perhaps keep the enum as SceneKind if necessary? 
2. Make some tangible progress
   1. Get initial server logic running

   
3. Refactor
   1. Make "Scene" an enum, where the variants are structs with the data they need to hold 
      1. Think more about this: https://www.reddit.com/r/rust/comments/buqgam/enum_variants_as_types/
      2. Consider using traits instead?
         - Yeah, use a trait "Draw" for example, so you can directly call scene.draw(terminal) instead of having to match against the different possible scenes
   2. Make the state of the logic state machine an enum instead of a struct, so the UI can use this same enum 

4. Implement stylesheet
   1. Define color constants in a file for this purpose

https://github.com/yasammez/nachricht 


- Send info to UI when server refreshes connection
- Should the backend to stuff like splitting files into chunks and hand them over to the server and also stitch them back together? 