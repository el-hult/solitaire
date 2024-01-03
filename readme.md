# solitaire

A simple Solitaire bot. It can play the game, but it's not very good at it.
I have tried to split the code into two parts: the game logic and the bot logic.
They communicate via a `SolitaireView` object, representing what a player can see.

There are may opportunities for optimization of the code. Some I have realized are

1. Minimize the allocations in creating the `SolitaireView` object. Can they hold references to the original GameState object instead, and make it faster that way? Initially, I only had accessors on the `GameState` and that was much faster to run, and a sore to program against. Can I constrict some middle ground?
2. Don't make stupid moves. I currently explore the game tree depth first with some simple heuristic to priotizie moves. In some cases (e.g. the first generated deal) this is very very inefficient. I can definitely do better! Some smarter search algorithm?


Use the code as you like, but you must refer back to me, and not close the source. 
Consider the licence CC-BY-SA 4.0.