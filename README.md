# MCTS for Go in PonyLang

v 0.1: full implementation of the Go rules, including ko detection. Supports complete random playouts.

# MCTS for Go in Rust

For comparison, [libego](https://github.com/lukaszlew/libego) does about 7.6 k playouts / second, or 0.13 ms per playout.

- 2015-07-11: bit of a false alarm, not quite that fast after fixing atari detection:
  |PT1.952084253S---PT1.964127458S---PT1.993767023S|, mean PT1.966033600S +- PT0.041374833S
  On the plus side, win rates are a more expected 45% now.
- 2015-07-11: giant speedup by switching to libego inspired pseudo-liberties!
  |PT1.661183028S---PT1.666016963S---PT1.695869809S|, mean PT1.668791982S +- PT0.030115182S
  That's about 6k playouts per second! Nearly the same speed as libego now.
- 2015-07-11: some slowdown after fixing ko-handling:
  |PT2.625271934S---PT2.635295411S---PT2.669489677S|, mean PT2.639132334S +- PT0.047698041S
- 2015-07-11: explicit list of empty vertices speeds up selecting random moves a bit:
  |PT2.400072437S---PT2.405261632S---PT2.422357665S|, mean PT2.406048004S +- PT0.019124224S
- 2015-07-11: another speedup from storing string membership as integer based linked lists, only a bit more than 2x slower than libego:
  |PT2.849937705S---PT2.850953776S---PT2.862720022S|, mean PT2.853414755S +- PT0.012636017S
  That's 3.5k rollouts per second!
- 2015-07-10: better random move selection cuts time in half:
  |PT3.781544956S---PT3.786354668S---PT3.797252693S|, mean PT3.787625374S +- PT0.014460112S
  (duration of 10k playouts)
  Only 3x slower than libego now.
- 2015-07-03: now with better statistics, down to below 0.8 ms:
  |PT7.698399799S---PT7.709610406S---PT7.750532471S|, mean PT7.713656318S +- PT0.043821621S
  Or 6 times slower than libego.
- 2015-07-01: optimized to 1.6 ms per playout
- 2015-07-01: playout time 2.1 ms, missing 3 kos in 1000 rollouts 
- 2015-06-29: playout time down to 10 ms, but missing 4 kos in 1000 rollouts
- 2015-06-29: playout time down to 20 ms
- 2015-06-28: playout time down to 150 ms
- 2015-06-06: 900 ms for a full playout on a 19x19 board
