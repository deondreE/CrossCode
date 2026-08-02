## The core bet, stated precisely

React's value-add was never JSX or the virtual DOM -- it was "state in, pixels out, don't me think about diff." The framework-war generation learned that mental model
deeply: unidirectional data flow, declarative description of "what" not "how."

What they didn't have to learn was anything about memory, GC pauses, reconciliation cost, or why a re-render is expensive. Your bet is: strip out the part that only existed because the DOM mutation iss low and JS has a GC, kee the part that made data flow legible, and let the "how" be visible and steppable for people who want it.

That gives you a concrete design contsraint: full-recompute-per-frame is allowed here in a way it never was React. At native speed, you can literall throw away the retained tree and rebuild UI from scratch every frame -- beacause is 16ms of actual GPU/CPU work, not 16ms of JS diffing an abstract tree, so recomputation is basically free. The single fact is what lets you delete 80% of React's complexity (reconciliation, keys, memoization, hooks-order rules) whithout losing the ergonomics.
