1. Refactor
   1. Put widgets into widget module file
   2. Update "UIMessage" to just be called "Message"
   3. Update "UIEvent" to just be called "Event"
   4. Make "Scene" an enum, where the variants are structs with the data they need to hold 
      1. Think more about this: https://www.reddit.com/r/rust/comments/buqgam/enum_variants_as_types/
      2. Consider using traits instead?

2. Implement Transmission stuff

3. Implement stylesheet
   1. Define color constants in a file for this purpose

