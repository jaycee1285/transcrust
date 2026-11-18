# John's vocabulary

One file for every speech surface: transcrust (laptop dictation), mopvox (phone
keyboard) and dayshade (Quick Settings tile). Edit it here or on the phone;
each app re-reads it.

One entry per canonical spelling:

    ## Apptrack
    kind: task
    heard: apt track, abtrack

- `kind` is one of habit · task · repo · person · place · term.
- `heard` is what the engines write instead, comma-separated. Optional.
- `description` gives the term's meaning for contextual decisions. Optional.
- `ordinary` describes the competing ordinary meaning. Optional.
- transcrust `--dec` uses these fields to judge each occurrence in context;
  aliases are candidates, not unconditional replacements. Other consumers keep
  their existing behavior and ignore metadata they do not use.

Who reads what:

- **transcrust and mopvox** take every heading whose kind is *not* habit or
  task into their phonetic pass. Habit and task names are short and ordinary
  (`Cigs` would claim every `six`), so dictation never sees them.
- **dayshade** takes a habit's `heard` list as aliases for the name slot only,
  never the number (`six: Cigs` must not turn "set six" into "set Cigs"), and
  every other `heard` list as bad-string rewrites that may only produce a name
  that exists today. `dayshade/pairs.md` stays the tap log beside this file.

## Kubernetes
kind: term

## Wayland
kind: term

## Svelte
kind: term

## Syncthing
kind: term

## transcrust
kind: repo
description: software project
ordinary: ordinary prose about material passing through a crust, not an application name
heard: trans crust

## Gambol
kind: person

heard: gamble
description: pet dog
ordinary: ordinary English about gambling or betting money

## Nell
kind: person
description: son's mom
ordinary: ordinary English about a knell, the sound of a bell marking a death
heard: Knell, knell

## Cigs
kind: habit
heard: sigs, six, cigarettes

## Void
kind: term
heard: Voice

## Rowan
kind: person
description: son
ordinary: a different person or an intentional initial, not the full name of the speaker's son
heard: Rowan

## Flo
kind: person
description: Cars movie character, runs the V8 Cafe
ordinary: ordinary flow, the movement of water, traffic or another substance
heard: flow

## Lightning McQueen
kind: person
description: Cars movie character, race car
ordinary: ordinary lightning, the electrical discharge during a storm
heard: lightning mcqueen

## Mater
kind: person
description: Cars movie character, tow truck
ordinary: a different person named Mader, or a mater meaning an animal or machine that mates or couples
heard: Mader, mater

## Bluey
kind: person
description: 6-year-old main character, blue heeler
ordinary: ordinary words about the color blue, rather than the TV character Bluey
heard: blue e

## Bingo
kind: person
description: Bluey's 4-year-old sister
ordinary: the ordinary game bingo or its name
heard: bingo, bin go

## Chili
kind: person
description: Bluey and Bingo's mom
ordinary: ordinary chili, the food or pepper, or chilly meaning cold
heard: chili, chilly

## Bandit
kind: person
description: Bluey and Bingo's dad
ordinary: an ordinary bandit, a robber or outlaw, or ordinary separate words band it
heard: bandit, band it

## Cars theme
kind: term
heard: car steam
