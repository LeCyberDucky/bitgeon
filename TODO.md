1. Fix that the UI doesn't sleep correctly between frames
2. Refactor
   1. Make "Scene" an enum, where the variants are structs with the data they need to hold 
      1. Think more about this: https://www.reddit.com/r/rust/comments/buqgam/enum_variants_as_types/
      2. Consider using traits instead?

3. Implement Transmission stuff

4. Implement stylesheet
   1. Define color constants in a file for this purpose

https://github.com/yasammez/nachricht 