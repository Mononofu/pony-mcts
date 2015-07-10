#ifndef MCTS_GO_GAME_H_
#define MCTS_GO_GAME_H_

#include <cstdint>

using Vertex = int16_t;

constexpr Vertex kPass = -1;

class GoGame {
public:
  explicit GoGame(int size);

};


#endif  // MCTS_GO_GAME_H_
