#import "template.typ": *
#show: setup.with(
  title: "BlastCap",
  snippet: "ROUGELITE TURNBASED DUNGEON CRAWLER",
  credits: [#emoji.bread],
  sidebar: "BlastCap Design Document",
  sidebar-color: rgb("#e49e64"),
)
#import "@preview/dashy-todo:0.0.2": *
#let inlineTodo(body) = box(stroke: red, inset: 2pt, [#box(outset: 2pt, fill: red, text(fill: white, "TODO")) #body]);

= High Level Concept/Design
== Working title
#inlineTodo[Your game’s title should communicate the gameplay and the style of the game.]

Work in progress title: *Blastcap*

== Concept statement
#inlineTodo[The game in a tweet: one or two sentences at most that say what the game is and why it’s fun.]


== Genre(s)
#inlineTodo[Single genre is clearer but often less interesting. Genre combinations can be risky. Beware of ‘tired’ genres.]


== Target audience
#inlineTodo[Motivations and relevant interests; potentially age, gender, etc.; and the desired ESRB rating for the game.]

== Unique Selling Points
#inlineTodo[Critically important. What makes your game stand out? How is it different from all other games?]

#pagebreak()
= Product Design
== Player Experience and Game POV
#inlineTodo[Who is the player? What is the setting? What is the fantasy the game grants the player? What emotions do
  you want the player to feel? What keeps the player engaged for the duration of their play?]

== Mood Board
#inlineTodo[Should contain pictures which can be used for artistic inspiration.]

== Visual and Audio Style
#inlineTodo[What is the “look and feel” of the game? How does this support the desired player’s experience? What
  concept art or reference art can you show to give the feel of the game?]

== Game World Fiction
See the Lore Compendium chapter for more details.

== Monetization
#inlineTodo[How will the game make money? Premium purchase? F2P? How do you justify this within the design?]

== Platform(s), Technology, and Scope (brief)
#inlineTodo[PC or mobile? Table or phone? 2D or 3D? Unity or Javascript? How long to make, and how big a team?
  How long to first-playable? How long to complete the game? Major risks?]

=== Target platform
The game would ideally target PC first. Console would be nice in the future if the game goes anywhere.

=== Engine
Game logic and server/client in Rust, with frontend written in C\# using Godot.

=== Time to create, and team size
Time to create is unknown, but probably long, as small team.

#pagebreak()
= Detailed & Game Systems Design
== Core Loops
#inlineTodo[How do game objects and the player’s actions form loops? Why is this engaging? How does this support
  player goals? What emergent results do you expect/hope to see? If F2P, where are the monetization points?]

== Objectives and Progression
#inlineTodo[How does the player move through the game, literally and figuratively, from tutorial to end? What are their
  short-term and long-term goals (explicit or implicit)? How do these support the game concept, style, and
  player-fantasy?]

== Game Systems
#inlineTodo[What systems are needed to make this game? Which ones are internal (simulation, etc.) and which does the
  player interact with?]

== Interactivity
#inlineTodo[How are different kinds of interactivity used? (Action/Feedback, ST Cog, LT Cog, Emotional, Social, Cultural)
  What is the player doing moment-by-moment? How does the player move through the world? How does
  physics/combat/etc. work? A clear, professional-looking sketch of the primary game UX is helpful.]

#pagebreak()
= Lore Compendium <LoreCompendium>
#inlineTodo[Start writing lore :)]
