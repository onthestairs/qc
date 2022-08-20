# `qc` - Quinian crosswords

A quinian crossword is a crossword which has one set of clues, but two potential solution grids. It was first introduced to me in Daniel Dennett's paper [_Quine in My Life_](https://www.jstor.org/stable/23025100). Dennett shows a 4x4 Quinian crossword he has manually constructed, and asks the reader if they can do better. This repo is an attempt to do so.

A rust program to make quine crosswords. There are two components to it -- the clue fetcher, and the generator. We use the fetcher to create a bank of clues, and the generator to use those clues to place them onto a grid such that a Quinian crossword is made.

## Clue fetcher

Clues are fetched (`./src/bin/fetch_clues.rs`) and stored in an sqlite database. There are two sources for the clues -- the Guardian quick crosswords, and the New York Times crossword. We simply store these as triples (source, surface, solution) in the database. We can then output (`./src/bin/export_clues.rs`) these clues to a CSV file which will be used by the generator.

## Generator

The generator will try to brute force a quinian crossword given the clues it is passed. 

Certain options can be specified to change the behaviour of the generator.
- The shape of the grid that we are searching for. The can either be `dense3`, `dense4`, `dense5` (for a full grid with no black cells) or `alternating5`, `alternating6`, `alternating7` (for grids with alternating white and black cells).
- It is possible to filter the clue list so that only words which have been used a certain number of times on wikipedia are kept in the corpus.
- Sometimes the placing of several solutions can lead to new solutions being generated without surfaces. The program will try to fill these surfaces in with a 'clue pair', but it can also leave the clue pair to be constructed by a human later. It is configurable as to how many of these un-clued solutions to allow.

Finally, we can print the found crosswords from the database using `./src/bin/print_qcs.rs`.
