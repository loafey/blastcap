#import "template.typ": *
#show: setup.with(
  title: "BlastCap",
  snippet: "ROUGELITE TURNBASED DUNGEON CRAWLER",
  credits: [#emoji.bread],
  sidebar: "BlastCap Design Document",
  sidebar-color: rgb("#e49e64"),
)
#import "@preview/dashy-todo:0.0.2": *
#import "@preview/finite:0.5.0": automaton, layout
#let inlineTodo(body) = box(stroke: red, inset: 2pt, [#box(outset: 2pt, fill: red, text(fill: white, "TODO")) #body]);

= High Level Concept/Design
== Working title
Work in progress title: *BlastCap*

Do not know if this name will stick, but thought it was a suitable action sounding
name.


== Concept statement
#inlineTodo[The game in a tweet: one or two sentences at most that say what the game is and why it’s fun.]


== Genre(s)
#inlineTodo[Single genre is clearer but often less interesting. Genre combinations can be risky. Beware of ‘tired’ genres.]

=== Turn based PVE
=== Rougelike
=== Dungeon Crawling


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
See the #link(<LoreCompendium>, "Lore Compendium") chapter for more details.

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
== Core Loop(s)
// #inlineTodo[How do game objects and the player’s actions form loops? Why is this engaging? How does this support
//   player goals? What emergent results do you expect/hope to see? If F2P, where are the monetization points?]
#figure(caption: "The Core Loop", supplement: "Figure", grid(
  // columns: (1fr, 1fr),
  align: center + horizon,
  gutter: 40pt,
  automaton(
    (
      ED: (PC: ""),
      PC: (CR: ""),
      CR: (LR: "", F: ""),
      LR: (CR: ""),
    ),
    // layout: layout.circular.with(spacing: 1.9),
    final: "F",
  ),
  // table(
  //   columns: (1fr, 1fr),
  //   table.cell(colspan: 2, align: center)[*Legend*],
  //   [*ED*], [Enter dungeon],
  //   [*CD*], [Clear dungeon],
  //   [*LD*], [Loot dungeon],
  //   [*PI$""_1$*], [Prepare Inventory],
  //   [*PI$""_2$*], [Prepare Inventory],
  //   [*F*], [Finish],
  // ),
))<CoreLoop>

The core loop of the game is relatively simple, and a normal game will play out much like
@CoreLoop specifies:
#let c(cont) = table.cell(align: center + horizon, cont)
#table(
  columns: (auto, auto, 1fr),
  c[*ED*],
  c[Enter Dungeon],
  [
    Enter a selected dungeon.
    The host player chooses the dungeon from a list of pre-configured dungeons.
  ],

  c[*PC*],
  c[Prepare Character],
  [
    Let players design a character that they will use through the dungeon.
    Here classes and starting abilities are chosen.
  ],

  c[*CR*],
  c[Clear Room],
  [
    Fight your way through a room containing traps and enemies.
  ],

  c[*LR*],
  c[Loot Room],
  [
    After a room has been cleared, you can start looting it.
    Loot is not unique so players have to divide it up between players.
    XP is also rewarded based on the rooms difficulty.

    Players also have time to modify their loadout, and if applicable,
    level up.

    When all players are ready, proceed to the next room, which will have
    an increased difficulty.
  ],

  c[*F*],
  c[Finish],
  [
    After the final room in dungeon has been cleared, finish the game and tally the score.
  ],
)

The hope here is that the gameplay loop will provide an engaging experience,
and allow for emergent gameplay by creating unique interactions
between classes, abilities, and items.

== RPG System<RPGSystem>
Player characters and non-player characters are all created using the game's
"unique" RPG system.

#inlineTodo[add more]

== Game Systems
#inlineTodo[What systems are needed to make this game? Which ones are internal (simulation, etc.) and which does the
  player interact with?]

== Objectives and Progression
#inlineTodo[How does the player move through the game, literally and figuratively, from tutorial to end? What are their
  short-term and long-term goals (explicit or implicit)? How do these support the game concept, style, and
  player-fantasy?]
=== Player Abilities<PlayerAbilities>
=== Player Loot<PlayerLoot>

== Interactivity
#inlineTodo[How are different kinds of interactivity used? (Action/Feedback, ST Cog, LT Cog, Emotional, Social, Cultural)
  What is the player doing moment-by-moment? How does the player move through the world? How does
  physics/combat/etc. work? A clear, professional-looking sketch of the primary game UX is helpful.]

#pagebreak()
= Lore Compendium <LoreCompendium>
#inlineTodo[Start scratching at some light lore :)]
