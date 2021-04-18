1. Refactor
   1. Make "Scene" an enum, where the variants are structs with the data they need to hold 
      1. Think more about this: https://www.reddit.com/r/rust/comments/buqgam/enum_variants_as_types/
      2. Consider using traits instead?
         - Yeah, use a trait "Draw" for example, so you can directly call scene.draw(terminal) instead of having to match against the different possible scenes
   2. Make the state of the logic state machine an enum instead of a struct, so the UI can use this same enum 

2. Implement Transmission stuff

3. Implement stylesheet
   1. Define color constants in a file for this purpose

https://github.com/yasammez/nachricht 