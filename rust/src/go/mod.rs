extern crate rand;

use rand::Rng;
use std::fmt;
use std::collections;
use std::mem;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Stone {
  Empty,
  Black,
  White,
  Border,
}

impl Stone {
  pub fn opponent(self) -> Stone {
    match self {
      Stone::Empty => Stone::Empty,
      Stone::Black => Stone::White,
      Stone::White => Stone::Black,
      Stone::Border => Stone::Border,
    }
  }
}

#[derive(Eq, Hash, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct Vertex(pub i16);

pub const PASS: Vertex = Vertex(-1);

const NEIGHBOURS: [[Vertex; 4]; 21 * 21] = [
  [Vertex(0 - 1), Vertex(0 + 1), Vertex(0 - 21), Vertex(0 + 21)],
  [Vertex(1 - 1), Vertex(1 + 1), Vertex(1 - 21), Vertex(1 + 21)],
  [Vertex(2 - 1), Vertex(2 + 1), Vertex(2 - 21), Vertex(2 + 21)],
  [Vertex(3 - 1), Vertex(3 + 1), Vertex(3 - 21), Vertex(3 + 21)],
  [Vertex(4 - 1), Vertex(4 + 1), Vertex(4 - 21), Vertex(4 + 21)],
  [Vertex(5 - 1), Vertex(5 + 1), Vertex(5 - 21), Vertex(5 + 21)],
  [Vertex(6 - 1), Vertex(6 + 1), Vertex(6 - 21), Vertex(6 + 21)],
  [Vertex(7 - 1), Vertex(7 + 1), Vertex(7 - 21), Vertex(7 + 21)],
  [Vertex(8 - 1), Vertex(8 + 1), Vertex(8 - 21), Vertex(8 + 21)],
  [Vertex(9 - 1), Vertex(9 + 1), Vertex(9 - 21), Vertex(9 + 21)],
  [Vertex(10 - 1), Vertex(10 + 1), Vertex(10 - 21), Vertex(10 + 21)],
  [Vertex(11 - 1), Vertex(11 + 1), Vertex(11 - 21), Vertex(11 + 21)],
  [Vertex(12 - 1), Vertex(12 + 1), Vertex(12 - 21), Vertex(12 + 21)],
  [Vertex(13 - 1), Vertex(13 + 1), Vertex(13 - 21), Vertex(13 + 21)],
  [Vertex(14 - 1), Vertex(14 + 1), Vertex(14 - 21), Vertex(14 + 21)],
  [Vertex(15 - 1), Vertex(15 + 1), Vertex(15 - 21), Vertex(15 + 21)],
  [Vertex(16 - 1), Vertex(16 + 1), Vertex(16 - 21), Vertex(16 + 21)],
  [Vertex(17 - 1), Vertex(17 + 1), Vertex(17 - 21), Vertex(17 + 21)],
  [Vertex(18 - 1), Vertex(18 + 1), Vertex(18 - 21), Vertex(18 + 21)],
  [Vertex(19 - 1), Vertex(19 + 1), Vertex(19 - 21), Vertex(19 + 21)],
  [Vertex(20 - 1), Vertex(20 + 1), Vertex(20 - 21), Vertex(20 + 21)],
  [Vertex(21 - 1), Vertex(21 + 1), Vertex(21 - 21), Vertex(21 + 21)],
  [Vertex(22 - 1), Vertex(22 + 1), Vertex(22 - 21), Vertex(22 + 21)],
  [Vertex(23 - 1), Vertex(23 + 1), Vertex(23 - 21), Vertex(23 + 21)],
  [Vertex(24 - 1), Vertex(24 + 1), Vertex(24 - 21), Vertex(24 + 21)],
  [Vertex(25 - 1), Vertex(25 + 1), Vertex(25 - 21), Vertex(25 + 21)],
  [Vertex(26 - 1), Vertex(26 + 1), Vertex(26 - 21), Vertex(26 + 21)],
  [Vertex(27 - 1), Vertex(27 + 1), Vertex(27 - 21), Vertex(27 + 21)],
  [Vertex(28 - 1), Vertex(28 + 1), Vertex(28 - 21), Vertex(28 + 21)],
  [Vertex(29 - 1), Vertex(29 + 1), Vertex(29 - 21), Vertex(29 + 21)],
  [Vertex(30 - 1), Vertex(30 + 1), Vertex(30 - 21), Vertex(30 + 21)],
  [Vertex(31 - 1), Vertex(31 + 1), Vertex(31 - 21), Vertex(31 + 21)],
  [Vertex(32 - 1), Vertex(32 + 1), Vertex(32 - 21), Vertex(32 + 21)],
  [Vertex(33 - 1), Vertex(33 + 1), Vertex(33 - 21), Vertex(33 + 21)],
  [Vertex(34 - 1), Vertex(34 + 1), Vertex(34 - 21), Vertex(34 + 21)],
  [Vertex(35 - 1), Vertex(35 + 1), Vertex(35 - 21), Vertex(35 + 21)],
  [Vertex(36 - 1), Vertex(36 + 1), Vertex(36 - 21), Vertex(36 + 21)],
  [Vertex(37 - 1), Vertex(37 + 1), Vertex(37 - 21), Vertex(37 + 21)],
  [Vertex(38 - 1), Vertex(38 + 1), Vertex(38 - 21), Vertex(38 + 21)],
  [Vertex(39 - 1), Vertex(39 + 1), Vertex(39 - 21), Vertex(39 + 21)],
  [Vertex(40 - 1), Vertex(40 + 1), Vertex(40 - 21), Vertex(40 + 21)],
  [Vertex(41 - 1), Vertex(41 + 1), Vertex(41 - 21), Vertex(41 + 21)],
  [Vertex(42 - 1), Vertex(42 + 1), Vertex(42 - 21), Vertex(42 + 21)],
  [Vertex(43 - 1), Vertex(43 + 1), Vertex(43 - 21), Vertex(43 + 21)],
  [Vertex(44 - 1), Vertex(44 + 1), Vertex(44 - 21), Vertex(44 + 21)],
  [Vertex(45 - 1), Vertex(45 + 1), Vertex(45 - 21), Vertex(45 + 21)],
  [Vertex(46 - 1), Vertex(46 + 1), Vertex(46 - 21), Vertex(46 + 21)],
  [Vertex(47 - 1), Vertex(47 + 1), Vertex(47 - 21), Vertex(47 + 21)],
  [Vertex(48 - 1), Vertex(48 + 1), Vertex(48 - 21), Vertex(48 + 21)],
  [Vertex(49 - 1), Vertex(49 + 1), Vertex(49 - 21), Vertex(49 + 21)],
  [Vertex(50 - 1), Vertex(50 + 1), Vertex(50 - 21), Vertex(50 + 21)],
  [Vertex(51 - 1), Vertex(51 + 1), Vertex(51 - 21), Vertex(51 + 21)],
  [Vertex(52 - 1), Vertex(52 + 1), Vertex(52 - 21), Vertex(52 + 21)],
  [Vertex(53 - 1), Vertex(53 + 1), Vertex(53 - 21), Vertex(53 + 21)],
  [Vertex(54 - 1), Vertex(54 + 1), Vertex(54 - 21), Vertex(54 + 21)],
  [Vertex(55 - 1), Vertex(55 + 1), Vertex(55 - 21), Vertex(55 + 21)],
  [Vertex(56 - 1), Vertex(56 + 1), Vertex(56 - 21), Vertex(56 + 21)],
  [Vertex(57 - 1), Vertex(57 + 1), Vertex(57 - 21), Vertex(57 + 21)],
  [Vertex(58 - 1), Vertex(58 + 1), Vertex(58 - 21), Vertex(58 + 21)],
  [Vertex(59 - 1), Vertex(59 + 1), Vertex(59 - 21), Vertex(59 + 21)],
  [Vertex(60 - 1), Vertex(60 + 1), Vertex(60 - 21), Vertex(60 + 21)],
  [Vertex(61 - 1), Vertex(61 + 1), Vertex(61 - 21), Vertex(61 + 21)],
  [Vertex(62 - 1), Vertex(62 + 1), Vertex(62 - 21), Vertex(62 + 21)],
  [Vertex(63 - 1), Vertex(63 + 1), Vertex(63 - 21), Vertex(63 + 21)],
  [Vertex(64 - 1), Vertex(64 + 1), Vertex(64 - 21), Vertex(64 + 21)],
  [Vertex(65 - 1), Vertex(65 + 1), Vertex(65 - 21), Vertex(65 + 21)],
  [Vertex(66 - 1), Vertex(66 + 1), Vertex(66 - 21), Vertex(66 + 21)],
  [Vertex(67 - 1), Vertex(67 + 1), Vertex(67 - 21), Vertex(67 + 21)],
  [Vertex(68 - 1), Vertex(68 + 1), Vertex(68 - 21), Vertex(68 + 21)],
  [Vertex(69 - 1), Vertex(69 + 1), Vertex(69 - 21), Vertex(69 + 21)],
  [Vertex(70 - 1), Vertex(70 + 1), Vertex(70 - 21), Vertex(70 + 21)],
  [Vertex(71 - 1), Vertex(71 + 1), Vertex(71 - 21), Vertex(71 + 21)],
  [Vertex(72 - 1), Vertex(72 + 1), Vertex(72 - 21), Vertex(72 + 21)],
  [Vertex(73 - 1), Vertex(73 + 1), Vertex(73 - 21), Vertex(73 + 21)],
  [Vertex(74 - 1), Vertex(74 + 1), Vertex(74 - 21), Vertex(74 + 21)],
  [Vertex(75 - 1), Vertex(75 + 1), Vertex(75 - 21), Vertex(75 + 21)],
  [Vertex(76 - 1), Vertex(76 + 1), Vertex(76 - 21), Vertex(76 + 21)],
  [Vertex(77 - 1), Vertex(77 + 1), Vertex(77 - 21), Vertex(77 + 21)],
  [Vertex(78 - 1), Vertex(78 + 1), Vertex(78 - 21), Vertex(78 + 21)],
  [Vertex(79 - 1), Vertex(79 + 1), Vertex(79 - 21), Vertex(79 + 21)],
  [Vertex(80 - 1), Vertex(80 + 1), Vertex(80 - 21), Vertex(80 + 21)],
  [Vertex(81 - 1), Vertex(81 + 1), Vertex(81 - 21), Vertex(81 + 21)],
  [Vertex(82 - 1), Vertex(82 + 1), Vertex(82 - 21), Vertex(82 + 21)],
  [Vertex(83 - 1), Vertex(83 + 1), Vertex(83 - 21), Vertex(83 + 21)],
  [Vertex(84 - 1), Vertex(84 + 1), Vertex(84 - 21), Vertex(84 + 21)],
  [Vertex(85 - 1), Vertex(85 + 1), Vertex(85 - 21), Vertex(85 + 21)],
  [Vertex(86 - 1), Vertex(86 + 1), Vertex(86 - 21), Vertex(86 + 21)],
  [Vertex(87 - 1), Vertex(87 + 1), Vertex(87 - 21), Vertex(87 + 21)],
  [Vertex(88 - 1), Vertex(88 + 1), Vertex(88 - 21), Vertex(88 + 21)],
  [Vertex(89 - 1), Vertex(89 + 1), Vertex(89 - 21), Vertex(89 + 21)],
  [Vertex(90 - 1), Vertex(90 + 1), Vertex(90 - 21), Vertex(90 + 21)],
  [Vertex(91 - 1), Vertex(91 + 1), Vertex(91 - 21), Vertex(91 + 21)],
  [Vertex(92 - 1), Vertex(92 + 1), Vertex(92 - 21), Vertex(92 + 21)],
  [Vertex(93 - 1), Vertex(93 + 1), Vertex(93 - 21), Vertex(93 + 21)],
  [Vertex(94 - 1), Vertex(94 + 1), Vertex(94 - 21), Vertex(94 + 21)],
  [Vertex(95 - 1), Vertex(95 + 1), Vertex(95 - 21), Vertex(95 + 21)],
  [Vertex(96 - 1), Vertex(96 + 1), Vertex(96 - 21), Vertex(96 + 21)],
  [Vertex(97 - 1), Vertex(97 + 1), Vertex(97 - 21), Vertex(97 + 21)],
  [Vertex(98 - 1), Vertex(98 + 1), Vertex(98 - 21), Vertex(98 + 21)],
  [Vertex(99 - 1), Vertex(99 + 1), Vertex(99 - 21), Vertex(99 + 21)],
  [Vertex(100 - 1), Vertex(100 + 1), Vertex(100 - 21), Vertex(100 + 21)],
  [Vertex(101 - 1), Vertex(101 + 1), Vertex(101 - 21), Vertex(101 + 21)],
  [Vertex(102 - 1), Vertex(102 + 1), Vertex(102 - 21), Vertex(102 + 21)],
  [Vertex(103 - 1), Vertex(103 + 1), Vertex(103 - 21), Vertex(103 + 21)],
  [Vertex(104 - 1), Vertex(104 + 1), Vertex(104 - 21), Vertex(104 + 21)],
  [Vertex(105 - 1), Vertex(105 + 1), Vertex(105 - 21), Vertex(105 + 21)],
  [Vertex(106 - 1), Vertex(106 + 1), Vertex(106 - 21), Vertex(106 + 21)],
  [Vertex(107 - 1), Vertex(107 + 1), Vertex(107 - 21), Vertex(107 + 21)],
  [Vertex(108 - 1), Vertex(108 + 1), Vertex(108 - 21), Vertex(108 + 21)],
  [Vertex(109 - 1), Vertex(109 + 1), Vertex(109 - 21), Vertex(109 + 21)],
  [Vertex(110 - 1), Vertex(110 + 1), Vertex(110 - 21), Vertex(110 + 21)],
  [Vertex(111 - 1), Vertex(111 + 1), Vertex(111 - 21), Vertex(111 + 21)],
  [Vertex(112 - 1), Vertex(112 + 1), Vertex(112 - 21), Vertex(112 + 21)],
  [Vertex(113 - 1), Vertex(113 + 1), Vertex(113 - 21), Vertex(113 + 21)],
  [Vertex(114 - 1), Vertex(114 + 1), Vertex(114 - 21), Vertex(114 + 21)],
  [Vertex(115 - 1), Vertex(115 + 1), Vertex(115 - 21), Vertex(115 + 21)],
  [Vertex(116 - 1), Vertex(116 + 1), Vertex(116 - 21), Vertex(116 + 21)],
  [Vertex(117 - 1), Vertex(117 + 1), Vertex(117 - 21), Vertex(117 + 21)],
  [Vertex(118 - 1), Vertex(118 + 1), Vertex(118 - 21), Vertex(118 + 21)],
  [Vertex(119 - 1), Vertex(119 + 1), Vertex(119 - 21), Vertex(119 + 21)],
  [Vertex(120 - 1), Vertex(120 + 1), Vertex(120 - 21), Vertex(120 + 21)],
  [Vertex(121 - 1), Vertex(121 + 1), Vertex(121 - 21), Vertex(121 + 21)],
  [Vertex(122 - 1), Vertex(122 + 1), Vertex(122 - 21), Vertex(122 + 21)],
  [Vertex(123 - 1), Vertex(123 + 1), Vertex(123 - 21), Vertex(123 + 21)],
  [Vertex(124 - 1), Vertex(124 + 1), Vertex(124 - 21), Vertex(124 + 21)],
  [Vertex(125 - 1), Vertex(125 + 1), Vertex(125 - 21), Vertex(125 + 21)],
  [Vertex(126 - 1), Vertex(126 + 1), Vertex(126 - 21), Vertex(126 + 21)],
  [Vertex(127 - 1), Vertex(127 + 1), Vertex(127 - 21), Vertex(127 + 21)],
  [Vertex(128 - 1), Vertex(128 + 1), Vertex(128 - 21), Vertex(128 + 21)],
  [Vertex(129 - 1), Vertex(129 + 1), Vertex(129 - 21), Vertex(129 + 21)],
  [Vertex(130 - 1), Vertex(130 + 1), Vertex(130 - 21), Vertex(130 + 21)],
  [Vertex(131 - 1), Vertex(131 + 1), Vertex(131 - 21), Vertex(131 + 21)],
  [Vertex(132 - 1), Vertex(132 + 1), Vertex(132 - 21), Vertex(132 + 21)],
  [Vertex(133 - 1), Vertex(133 + 1), Vertex(133 - 21), Vertex(133 + 21)],
  [Vertex(134 - 1), Vertex(134 + 1), Vertex(134 - 21), Vertex(134 + 21)],
  [Vertex(135 - 1), Vertex(135 + 1), Vertex(135 - 21), Vertex(135 + 21)],
  [Vertex(136 - 1), Vertex(136 + 1), Vertex(136 - 21), Vertex(136 + 21)],
  [Vertex(137 - 1), Vertex(137 + 1), Vertex(137 - 21), Vertex(137 + 21)],
  [Vertex(138 - 1), Vertex(138 + 1), Vertex(138 - 21), Vertex(138 + 21)],
  [Vertex(139 - 1), Vertex(139 + 1), Vertex(139 - 21), Vertex(139 + 21)],
  [Vertex(140 - 1), Vertex(140 + 1), Vertex(140 - 21), Vertex(140 + 21)],
  [Vertex(141 - 1), Vertex(141 + 1), Vertex(141 - 21), Vertex(141 + 21)],
  [Vertex(142 - 1), Vertex(142 + 1), Vertex(142 - 21), Vertex(142 + 21)],
  [Vertex(143 - 1), Vertex(143 + 1), Vertex(143 - 21), Vertex(143 + 21)],
  [Vertex(144 - 1), Vertex(144 + 1), Vertex(144 - 21), Vertex(144 + 21)],
  [Vertex(145 - 1), Vertex(145 + 1), Vertex(145 - 21), Vertex(145 + 21)],
  [Vertex(146 - 1), Vertex(146 + 1), Vertex(146 - 21), Vertex(146 + 21)],
  [Vertex(147 - 1), Vertex(147 + 1), Vertex(147 - 21), Vertex(147 + 21)],
  [Vertex(148 - 1), Vertex(148 + 1), Vertex(148 - 21), Vertex(148 + 21)],
  [Vertex(149 - 1), Vertex(149 + 1), Vertex(149 - 21), Vertex(149 + 21)],
  [Vertex(150 - 1), Vertex(150 + 1), Vertex(150 - 21), Vertex(150 + 21)],
  [Vertex(151 - 1), Vertex(151 + 1), Vertex(151 - 21), Vertex(151 + 21)],
  [Vertex(152 - 1), Vertex(152 + 1), Vertex(152 - 21), Vertex(152 + 21)],
  [Vertex(153 - 1), Vertex(153 + 1), Vertex(153 - 21), Vertex(153 + 21)],
  [Vertex(154 - 1), Vertex(154 + 1), Vertex(154 - 21), Vertex(154 + 21)],
  [Vertex(155 - 1), Vertex(155 + 1), Vertex(155 - 21), Vertex(155 + 21)],
  [Vertex(156 - 1), Vertex(156 + 1), Vertex(156 - 21), Vertex(156 + 21)],
  [Vertex(157 - 1), Vertex(157 + 1), Vertex(157 - 21), Vertex(157 + 21)],
  [Vertex(158 - 1), Vertex(158 + 1), Vertex(158 - 21), Vertex(158 + 21)],
  [Vertex(159 - 1), Vertex(159 + 1), Vertex(159 - 21), Vertex(159 + 21)],
  [Vertex(160 - 1), Vertex(160 + 1), Vertex(160 - 21), Vertex(160 + 21)],
  [Vertex(161 - 1), Vertex(161 + 1), Vertex(161 - 21), Vertex(161 + 21)],
  [Vertex(162 - 1), Vertex(162 + 1), Vertex(162 - 21), Vertex(162 + 21)],
  [Vertex(163 - 1), Vertex(163 + 1), Vertex(163 - 21), Vertex(163 + 21)],
  [Vertex(164 - 1), Vertex(164 + 1), Vertex(164 - 21), Vertex(164 + 21)],
  [Vertex(165 - 1), Vertex(165 + 1), Vertex(165 - 21), Vertex(165 + 21)],
  [Vertex(166 - 1), Vertex(166 + 1), Vertex(166 - 21), Vertex(166 + 21)],
  [Vertex(167 - 1), Vertex(167 + 1), Vertex(167 - 21), Vertex(167 + 21)],
  [Vertex(168 - 1), Vertex(168 + 1), Vertex(168 - 21), Vertex(168 + 21)],
  [Vertex(169 - 1), Vertex(169 + 1), Vertex(169 - 21), Vertex(169 + 21)],
  [Vertex(170 - 1), Vertex(170 + 1), Vertex(170 - 21), Vertex(170 + 21)],
  [Vertex(171 - 1), Vertex(171 + 1), Vertex(171 - 21), Vertex(171 + 21)],
  [Vertex(172 - 1), Vertex(172 + 1), Vertex(172 - 21), Vertex(172 + 21)],
  [Vertex(173 - 1), Vertex(173 + 1), Vertex(173 - 21), Vertex(173 + 21)],
  [Vertex(174 - 1), Vertex(174 + 1), Vertex(174 - 21), Vertex(174 + 21)],
  [Vertex(175 - 1), Vertex(175 + 1), Vertex(175 - 21), Vertex(175 + 21)],
  [Vertex(176 - 1), Vertex(176 + 1), Vertex(176 - 21), Vertex(176 + 21)],
  [Vertex(177 - 1), Vertex(177 + 1), Vertex(177 - 21), Vertex(177 + 21)],
  [Vertex(178 - 1), Vertex(178 + 1), Vertex(178 - 21), Vertex(178 + 21)],
  [Vertex(179 - 1), Vertex(179 + 1), Vertex(179 - 21), Vertex(179 + 21)],
  [Vertex(180 - 1), Vertex(180 + 1), Vertex(180 - 21), Vertex(180 + 21)],
  [Vertex(181 - 1), Vertex(181 + 1), Vertex(181 - 21), Vertex(181 + 21)],
  [Vertex(182 - 1), Vertex(182 + 1), Vertex(182 - 21), Vertex(182 + 21)],
  [Vertex(183 - 1), Vertex(183 + 1), Vertex(183 - 21), Vertex(183 + 21)],
  [Vertex(184 - 1), Vertex(184 + 1), Vertex(184 - 21), Vertex(184 + 21)],
  [Vertex(185 - 1), Vertex(185 + 1), Vertex(185 - 21), Vertex(185 + 21)],
  [Vertex(186 - 1), Vertex(186 + 1), Vertex(186 - 21), Vertex(186 + 21)],
  [Vertex(187 - 1), Vertex(187 + 1), Vertex(187 - 21), Vertex(187 + 21)],
  [Vertex(188 - 1), Vertex(188 + 1), Vertex(188 - 21), Vertex(188 + 21)],
  [Vertex(189 - 1), Vertex(189 + 1), Vertex(189 - 21), Vertex(189 + 21)],
  [Vertex(190 - 1), Vertex(190 + 1), Vertex(190 - 21), Vertex(190 + 21)],
  [Vertex(191 - 1), Vertex(191 + 1), Vertex(191 - 21), Vertex(191 + 21)],
  [Vertex(192 - 1), Vertex(192 + 1), Vertex(192 - 21), Vertex(192 + 21)],
  [Vertex(193 - 1), Vertex(193 + 1), Vertex(193 - 21), Vertex(193 + 21)],
  [Vertex(194 - 1), Vertex(194 + 1), Vertex(194 - 21), Vertex(194 + 21)],
  [Vertex(195 - 1), Vertex(195 + 1), Vertex(195 - 21), Vertex(195 + 21)],
  [Vertex(196 - 1), Vertex(196 + 1), Vertex(196 - 21), Vertex(196 + 21)],
  [Vertex(197 - 1), Vertex(197 + 1), Vertex(197 - 21), Vertex(197 + 21)],
  [Vertex(198 - 1), Vertex(198 + 1), Vertex(198 - 21), Vertex(198 + 21)],
  [Vertex(199 - 1), Vertex(199 + 1), Vertex(199 - 21), Vertex(199 + 21)],
  [Vertex(200 - 1), Vertex(200 + 1), Vertex(200 - 21), Vertex(200 + 21)],
  [Vertex(201 - 1), Vertex(201 + 1), Vertex(201 - 21), Vertex(201 + 21)],
  [Vertex(202 - 1), Vertex(202 + 1), Vertex(202 - 21), Vertex(202 + 21)],
  [Vertex(203 - 1), Vertex(203 + 1), Vertex(203 - 21), Vertex(203 + 21)],
  [Vertex(204 - 1), Vertex(204 + 1), Vertex(204 - 21), Vertex(204 + 21)],
  [Vertex(205 - 1), Vertex(205 + 1), Vertex(205 - 21), Vertex(205 + 21)],
  [Vertex(206 - 1), Vertex(206 + 1), Vertex(206 - 21), Vertex(206 + 21)],
  [Vertex(207 - 1), Vertex(207 + 1), Vertex(207 - 21), Vertex(207 + 21)],
  [Vertex(208 - 1), Vertex(208 + 1), Vertex(208 - 21), Vertex(208 + 21)],
  [Vertex(209 - 1), Vertex(209 + 1), Vertex(209 - 21), Vertex(209 + 21)],
  [Vertex(210 - 1), Vertex(210 + 1), Vertex(210 - 21), Vertex(210 + 21)],
  [Vertex(211 - 1), Vertex(211 + 1), Vertex(211 - 21), Vertex(211 + 21)],
  [Vertex(212 - 1), Vertex(212 + 1), Vertex(212 - 21), Vertex(212 + 21)],
  [Vertex(213 - 1), Vertex(213 + 1), Vertex(213 - 21), Vertex(213 + 21)],
  [Vertex(214 - 1), Vertex(214 + 1), Vertex(214 - 21), Vertex(214 + 21)],
  [Vertex(215 - 1), Vertex(215 + 1), Vertex(215 - 21), Vertex(215 + 21)],
  [Vertex(216 - 1), Vertex(216 + 1), Vertex(216 - 21), Vertex(216 + 21)],
  [Vertex(217 - 1), Vertex(217 + 1), Vertex(217 - 21), Vertex(217 + 21)],
  [Vertex(218 - 1), Vertex(218 + 1), Vertex(218 - 21), Vertex(218 + 21)],
  [Vertex(219 - 1), Vertex(219 + 1), Vertex(219 - 21), Vertex(219 + 21)],
  [Vertex(220 - 1), Vertex(220 + 1), Vertex(220 - 21), Vertex(220 + 21)],
  [Vertex(221 - 1), Vertex(221 + 1), Vertex(221 - 21), Vertex(221 + 21)],
  [Vertex(222 - 1), Vertex(222 + 1), Vertex(222 - 21), Vertex(222 + 21)],
  [Vertex(223 - 1), Vertex(223 + 1), Vertex(223 - 21), Vertex(223 + 21)],
  [Vertex(224 - 1), Vertex(224 + 1), Vertex(224 - 21), Vertex(224 + 21)],
  [Vertex(225 - 1), Vertex(225 + 1), Vertex(225 - 21), Vertex(225 + 21)],
  [Vertex(226 - 1), Vertex(226 + 1), Vertex(226 - 21), Vertex(226 + 21)],
  [Vertex(227 - 1), Vertex(227 + 1), Vertex(227 - 21), Vertex(227 + 21)],
  [Vertex(228 - 1), Vertex(228 + 1), Vertex(228 - 21), Vertex(228 + 21)],
  [Vertex(229 - 1), Vertex(229 + 1), Vertex(229 - 21), Vertex(229 + 21)],
  [Vertex(230 - 1), Vertex(230 + 1), Vertex(230 - 21), Vertex(230 + 21)],
  [Vertex(231 - 1), Vertex(231 + 1), Vertex(231 - 21), Vertex(231 + 21)],
  [Vertex(232 - 1), Vertex(232 + 1), Vertex(232 - 21), Vertex(232 + 21)],
  [Vertex(233 - 1), Vertex(233 + 1), Vertex(233 - 21), Vertex(233 + 21)],
  [Vertex(234 - 1), Vertex(234 + 1), Vertex(234 - 21), Vertex(234 + 21)],
  [Vertex(235 - 1), Vertex(235 + 1), Vertex(235 - 21), Vertex(235 + 21)],
  [Vertex(236 - 1), Vertex(236 + 1), Vertex(236 - 21), Vertex(236 + 21)],
  [Vertex(237 - 1), Vertex(237 + 1), Vertex(237 - 21), Vertex(237 + 21)],
  [Vertex(238 - 1), Vertex(238 + 1), Vertex(238 - 21), Vertex(238 + 21)],
  [Vertex(239 - 1), Vertex(239 + 1), Vertex(239 - 21), Vertex(239 + 21)],
  [Vertex(240 - 1), Vertex(240 + 1), Vertex(240 - 21), Vertex(240 + 21)],
  [Vertex(241 - 1), Vertex(241 + 1), Vertex(241 - 21), Vertex(241 + 21)],
  [Vertex(242 - 1), Vertex(242 + 1), Vertex(242 - 21), Vertex(242 + 21)],
  [Vertex(243 - 1), Vertex(243 + 1), Vertex(243 - 21), Vertex(243 + 21)],
  [Vertex(244 - 1), Vertex(244 + 1), Vertex(244 - 21), Vertex(244 + 21)],
  [Vertex(245 - 1), Vertex(245 + 1), Vertex(245 - 21), Vertex(245 + 21)],
  [Vertex(246 - 1), Vertex(246 + 1), Vertex(246 - 21), Vertex(246 + 21)],
  [Vertex(247 - 1), Vertex(247 + 1), Vertex(247 - 21), Vertex(247 + 21)],
  [Vertex(248 - 1), Vertex(248 + 1), Vertex(248 - 21), Vertex(248 + 21)],
  [Vertex(249 - 1), Vertex(249 + 1), Vertex(249 - 21), Vertex(249 + 21)],
  [Vertex(250 - 1), Vertex(250 + 1), Vertex(250 - 21), Vertex(250 + 21)],
  [Vertex(251 - 1), Vertex(251 + 1), Vertex(251 - 21), Vertex(251 + 21)],
  [Vertex(252 - 1), Vertex(252 + 1), Vertex(252 - 21), Vertex(252 + 21)],
  [Vertex(253 - 1), Vertex(253 + 1), Vertex(253 - 21), Vertex(253 + 21)],
  [Vertex(254 - 1), Vertex(254 + 1), Vertex(254 - 21), Vertex(254 + 21)],
  [Vertex(255 - 1), Vertex(255 + 1), Vertex(255 - 21), Vertex(255 + 21)],
  [Vertex(256 - 1), Vertex(256 + 1), Vertex(256 - 21), Vertex(256 + 21)],
  [Vertex(257 - 1), Vertex(257 + 1), Vertex(257 - 21), Vertex(257 + 21)],
  [Vertex(258 - 1), Vertex(258 + 1), Vertex(258 - 21), Vertex(258 + 21)],
  [Vertex(259 - 1), Vertex(259 + 1), Vertex(259 - 21), Vertex(259 + 21)],
  [Vertex(260 - 1), Vertex(260 + 1), Vertex(260 - 21), Vertex(260 + 21)],
  [Vertex(261 - 1), Vertex(261 + 1), Vertex(261 - 21), Vertex(261 + 21)],
  [Vertex(262 - 1), Vertex(262 + 1), Vertex(262 - 21), Vertex(262 + 21)],
  [Vertex(263 - 1), Vertex(263 + 1), Vertex(263 - 21), Vertex(263 + 21)],
  [Vertex(264 - 1), Vertex(264 + 1), Vertex(264 - 21), Vertex(264 + 21)],
  [Vertex(265 - 1), Vertex(265 + 1), Vertex(265 - 21), Vertex(265 + 21)],
  [Vertex(266 - 1), Vertex(266 + 1), Vertex(266 - 21), Vertex(266 + 21)],
  [Vertex(267 - 1), Vertex(267 + 1), Vertex(267 - 21), Vertex(267 + 21)],
  [Vertex(268 - 1), Vertex(268 + 1), Vertex(268 - 21), Vertex(268 + 21)],
  [Vertex(269 - 1), Vertex(269 + 1), Vertex(269 - 21), Vertex(269 + 21)],
  [Vertex(270 - 1), Vertex(270 + 1), Vertex(270 - 21), Vertex(270 + 21)],
  [Vertex(271 - 1), Vertex(271 + 1), Vertex(271 - 21), Vertex(271 + 21)],
  [Vertex(272 - 1), Vertex(272 + 1), Vertex(272 - 21), Vertex(272 + 21)],
  [Vertex(273 - 1), Vertex(273 + 1), Vertex(273 - 21), Vertex(273 + 21)],
  [Vertex(274 - 1), Vertex(274 + 1), Vertex(274 - 21), Vertex(274 + 21)],
  [Vertex(275 - 1), Vertex(275 + 1), Vertex(275 - 21), Vertex(275 + 21)],
  [Vertex(276 - 1), Vertex(276 + 1), Vertex(276 - 21), Vertex(276 + 21)],
  [Vertex(277 - 1), Vertex(277 + 1), Vertex(277 - 21), Vertex(277 + 21)],
  [Vertex(278 - 1), Vertex(278 + 1), Vertex(278 - 21), Vertex(278 + 21)],
  [Vertex(279 - 1), Vertex(279 + 1), Vertex(279 - 21), Vertex(279 + 21)],
  [Vertex(280 - 1), Vertex(280 + 1), Vertex(280 - 21), Vertex(280 + 21)],
  [Vertex(281 - 1), Vertex(281 + 1), Vertex(281 - 21), Vertex(281 + 21)],
  [Vertex(282 - 1), Vertex(282 + 1), Vertex(282 - 21), Vertex(282 + 21)],
  [Vertex(283 - 1), Vertex(283 + 1), Vertex(283 - 21), Vertex(283 + 21)],
  [Vertex(284 - 1), Vertex(284 + 1), Vertex(284 - 21), Vertex(284 + 21)],
  [Vertex(285 - 1), Vertex(285 + 1), Vertex(285 - 21), Vertex(285 + 21)],
  [Vertex(286 - 1), Vertex(286 + 1), Vertex(286 - 21), Vertex(286 + 21)],
  [Vertex(287 - 1), Vertex(287 + 1), Vertex(287 - 21), Vertex(287 + 21)],
  [Vertex(288 - 1), Vertex(288 + 1), Vertex(288 - 21), Vertex(288 + 21)],
  [Vertex(289 - 1), Vertex(289 + 1), Vertex(289 - 21), Vertex(289 + 21)],
  [Vertex(290 - 1), Vertex(290 + 1), Vertex(290 - 21), Vertex(290 + 21)],
  [Vertex(291 - 1), Vertex(291 + 1), Vertex(291 - 21), Vertex(291 + 21)],
  [Vertex(292 - 1), Vertex(292 + 1), Vertex(292 - 21), Vertex(292 + 21)],
  [Vertex(293 - 1), Vertex(293 + 1), Vertex(293 - 21), Vertex(293 + 21)],
  [Vertex(294 - 1), Vertex(294 + 1), Vertex(294 - 21), Vertex(294 + 21)],
  [Vertex(295 - 1), Vertex(295 + 1), Vertex(295 - 21), Vertex(295 + 21)],
  [Vertex(296 - 1), Vertex(296 + 1), Vertex(296 - 21), Vertex(296 + 21)],
  [Vertex(297 - 1), Vertex(297 + 1), Vertex(297 - 21), Vertex(297 + 21)],
  [Vertex(298 - 1), Vertex(298 + 1), Vertex(298 - 21), Vertex(298 + 21)],
  [Vertex(299 - 1), Vertex(299 + 1), Vertex(299 - 21), Vertex(299 + 21)],
  [Vertex(300 - 1), Vertex(300 + 1), Vertex(300 - 21), Vertex(300 + 21)],
  [Vertex(301 - 1), Vertex(301 + 1), Vertex(301 - 21), Vertex(301 + 21)],
  [Vertex(302 - 1), Vertex(302 + 1), Vertex(302 - 21), Vertex(302 + 21)],
  [Vertex(303 - 1), Vertex(303 + 1), Vertex(303 - 21), Vertex(303 + 21)],
  [Vertex(304 - 1), Vertex(304 + 1), Vertex(304 - 21), Vertex(304 + 21)],
  [Vertex(305 - 1), Vertex(305 + 1), Vertex(305 - 21), Vertex(305 + 21)],
  [Vertex(306 - 1), Vertex(306 + 1), Vertex(306 - 21), Vertex(306 + 21)],
  [Vertex(307 - 1), Vertex(307 + 1), Vertex(307 - 21), Vertex(307 + 21)],
  [Vertex(308 - 1), Vertex(308 + 1), Vertex(308 - 21), Vertex(308 + 21)],
  [Vertex(309 - 1), Vertex(309 + 1), Vertex(309 - 21), Vertex(309 + 21)],
  [Vertex(310 - 1), Vertex(310 + 1), Vertex(310 - 21), Vertex(310 + 21)],
  [Vertex(311 - 1), Vertex(311 + 1), Vertex(311 - 21), Vertex(311 + 21)],
  [Vertex(312 - 1), Vertex(312 + 1), Vertex(312 - 21), Vertex(312 + 21)],
  [Vertex(313 - 1), Vertex(313 + 1), Vertex(313 - 21), Vertex(313 + 21)],
  [Vertex(314 - 1), Vertex(314 + 1), Vertex(314 - 21), Vertex(314 + 21)],
  [Vertex(315 - 1), Vertex(315 + 1), Vertex(315 - 21), Vertex(315 + 21)],
  [Vertex(316 - 1), Vertex(316 + 1), Vertex(316 - 21), Vertex(316 + 21)],
  [Vertex(317 - 1), Vertex(317 + 1), Vertex(317 - 21), Vertex(317 + 21)],
  [Vertex(318 - 1), Vertex(318 + 1), Vertex(318 - 21), Vertex(318 + 21)],
  [Vertex(319 - 1), Vertex(319 + 1), Vertex(319 - 21), Vertex(319 + 21)],
  [Vertex(320 - 1), Vertex(320 + 1), Vertex(320 - 21), Vertex(320 + 21)],
  [Vertex(321 - 1), Vertex(321 + 1), Vertex(321 - 21), Vertex(321 + 21)],
  [Vertex(322 - 1), Vertex(322 + 1), Vertex(322 - 21), Vertex(322 + 21)],
  [Vertex(323 - 1), Vertex(323 + 1), Vertex(323 - 21), Vertex(323 + 21)],
  [Vertex(324 - 1), Vertex(324 + 1), Vertex(324 - 21), Vertex(324 + 21)],
  [Vertex(325 - 1), Vertex(325 + 1), Vertex(325 - 21), Vertex(325 + 21)],
  [Vertex(326 - 1), Vertex(326 + 1), Vertex(326 - 21), Vertex(326 + 21)],
  [Vertex(327 - 1), Vertex(327 + 1), Vertex(327 - 21), Vertex(327 + 21)],
  [Vertex(328 - 1), Vertex(328 + 1), Vertex(328 - 21), Vertex(328 + 21)],
  [Vertex(329 - 1), Vertex(329 + 1), Vertex(329 - 21), Vertex(329 + 21)],
  [Vertex(330 - 1), Vertex(330 + 1), Vertex(330 - 21), Vertex(330 + 21)],
  [Vertex(331 - 1), Vertex(331 + 1), Vertex(331 - 21), Vertex(331 + 21)],
  [Vertex(332 - 1), Vertex(332 + 1), Vertex(332 - 21), Vertex(332 + 21)],
  [Vertex(333 - 1), Vertex(333 + 1), Vertex(333 - 21), Vertex(333 + 21)],
  [Vertex(334 - 1), Vertex(334 + 1), Vertex(334 - 21), Vertex(334 + 21)],
  [Vertex(335 - 1), Vertex(335 + 1), Vertex(335 - 21), Vertex(335 + 21)],
  [Vertex(336 - 1), Vertex(336 + 1), Vertex(336 - 21), Vertex(336 + 21)],
  [Vertex(337 - 1), Vertex(337 + 1), Vertex(337 - 21), Vertex(337 + 21)],
  [Vertex(338 - 1), Vertex(338 + 1), Vertex(338 - 21), Vertex(338 + 21)],
  [Vertex(339 - 1), Vertex(339 + 1), Vertex(339 - 21), Vertex(339 + 21)],
  [Vertex(340 - 1), Vertex(340 + 1), Vertex(340 - 21), Vertex(340 + 21)],
  [Vertex(341 - 1), Vertex(341 + 1), Vertex(341 - 21), Vertex(341 + 21)],
  [Vertex(342 - 1), Vertex(342 + 1), Vertex(342 - 21), Vertex(342 + 21)],
  [Vertex(343 - 1), Vertex(343 + 1), Vertex(343 - 21), Vertex(343 + 21)],
  [Vertex(344 - 1), Vertex(344 + 1), Vertex(344 - 21), Vertex(344 + 21)],
  [Vertex(345 - 1), Vertex(345 + 1), Vertex(345 - 21), Vertex(345 + 21)],
  [Vertex(346 - 1), Vertex(346 + 1), Vertex(346 - 21), Vertex(346 + 21)],
  [Vertex(347 - 1), Vertex(347 + 1), Vertex(347 - 21), Vertex(347 + 21)],
  [Vertex(348 - 1), Vertex(348 + 1), Vertex(348 - 21), Vertex(348 + 21)],
  [Vertex(349 - 1), Vertex(349 + 1), Vertex(349 - 21), Vertex(349 + 21)],
  [Vertex(350 - 1), Vertex(350 + 1), Vertex(350 - 21), Vertex(350 + 21)],
  [Vertex(351 - 1), Vertex(351 + 1), Vertex(351 - 21), Vertex(351 + 21)],
  [Vertex(352 - 1), Vertex(352 + 1), Vertex(352 - 21), Vertex(352 + 21)],
  [Vertex(353 - 1), Vertex(353 + 1), Vertex(353 - 21), Vertex(353 + 21)],
  [Vertex(354 - 1), Vertex(354 + 1), Vertex(354 - 21), Vertex(354 + 21)],
  [Vertex(355 - 1), Vertex(355 + 1), Vertex(355 - 21), Vertex(355 + 21)],
  [Vertex(356 - 1), Vertex(356 + 1), Vertex(356 - 21), Vertex(356 + 21)],
  [Vertex(357 - 1), Vertex(357 + 1), Vertex(357 - 21), Vertex(357 + 21)],
  [Vertex(358 - 1), Vertex(358 + 1), Vertex(358 - 21), Vertex(358 + 21)],
  [Vertex(359 - 1), Vertex(359 + 1), Vertex(359 - 21), Vertex(359 + 21)],
  [Vertex(360 - 1), Vertex(360 + 1), Vertex(360 - 21), Vertex(360 + 21)],
  [Vertex(361 - 1), Vertex(361 + 1), Vertex(361 - 21), Vertex(361 + 21)],
  [Vertex(362 - 1), Vertex(362 + 1), Vertex(362 - 21), Vertex(362 + 21)],
  [Vertex(363 - 1), Vertex(363 + 1), Vertex(363 - 21), Vertex(363 + 21)],
  [Vertex(364 - 1), Vertex(364 + 1), Vertex(364 - 21), Vertex(364 + 21)],
  [Vertex(365 - 1), Vertex(365 + 1), Vertex(365 - 21), Vertex(365 + 21)],
  [Vertex(366 - 1), Vertex(366 + 1), Vertex(366 - 21), Vertex(366 + 21)],
  [Vertex(367 - 1), Vertex(367 + 1), Vertex(367 - 21), Vertex(367 + 21)],
  [Vertex(368 - 1), Vertex(368 + 1), Vertex(368 - 21), Vertex(368 + 21)],
  [Vertex(369 - 1), Vertex(369 + 1), Vertex(369 - 21), Vertex(369 + 21)],
  [Vertex(370 - 1), Vertex(370 + 1), Vertex(370 - 21), Vertex(370 + 21)],
  [Vertex(371 - 1), Vertex(371 + 1), Vertex(371 - 21), Vertex(371 + 21)],
  [Vertex(372 - 1), Vertex(372 + 1), Vertex(372 - 21), Vertex(372 + 21)],
  [Vertex(373 - 1), Vertex(373 + 1), Vertex(373 - 21), Vertex(373 + 21)],
  [Vertex(374 - 1), Vertex(374 + 1), Vertex(374 - 21), Vertex(374 + 21)],
  [Vertex(375 - 1), Vertex(375 + 1), Vertex(375 - 21), Vertex(375 + 21)],
  [Vertex(376 - 1), Vertex(376 + 1), Vertex(376 - 21), Vertex(376 + 21)],
  [Vertex(377 - 1), Vertex(377 + 1), Vertex(377 - 21), Vertex(377 + 21)],
  [Vertex(378 - 1), Vertex(378 + 1), Vertex(378 - 21), Vertex(378 + 21)],
  [Vertex(379 - 1), Vertex(379 + 1), Vertex(379 - 21), Vertex(379 + 21)],
  [Vertex(380 - 1), Vertex(380 + 1), Vertex(380 - 21), Vertex(380 + 21)],
  [Vertex(381 - 1), Vertex(381 + 1), Vertex(381 - 21), Vertex(381 + 21)],
  [Vertex(382 - 1), Vertex(382 + 1), Vertex(382 - 21), Vertex(382 + 21)],
  [Vertex(383 - 1), Vertex(383 + 1), Vertex(383 - 21), Vertex(383 + 21)],
  [Vertex(384 - 1), Vertex(384 + 1), Vertex(384 - 21), Vertex(384 + 21)],
  [Vertex(385 - 1), Vertex(385 + 1), Vertex(385 - 21), Vertex(385 + 21)],
  [Vertex(386 - 1), Vertex(386 + 1), Vertex(386 - 21), Vertex(386 + 21)],
  [Vertex(387 - 1), Vertex(387 + 1), Vertex(387 - 21), Vertex(387 + 21)],
  [Vertex(388 - 1), Vertex(388 + 1), Vertex(388 - 21), Vertex(388 + 21)],
  [Vertex(389 - 1), Vertex(389 + 1), Vertex(389 - 21), Vertex(389 + 21)],
  [Vertex(390 - 1), Vertex(390 + 1), Vertex(390 - 21), Vertex(390 + 21)],
  [Vertex(391 - 1), Vertex(391 + 1), Vertex(391 - 21), Vertex(391 + 21)],
  [Vertex(392 - 1), Vertex(392 + 1), Vertex(392 - 21), Vertex(392 + 21)],
  [Vertex(393 - 1), Vertex(393 + 1), Vertex(393 - 21), Vertex(393 + 21)],
  [Vertex(394 - 1), Vertex(394 + 1), Vertex(394 - 21), Vertex(394 + 21)],
  [Vertex(395 - 1), Vertex(395 + 1), Vertex(395 - 21), Vertex(395 + 21)],
  [Vertex(396 - 1), Vertex(396 + 1), Vertex(396 - 21), Vertex(396 + 21)],
  [Vertex(397 - 1), Vertex(397 + 1), Vertex(397 - 21), Vertex(397 + 21)],
  [Vertex(398 - 1), Vertex(398 + 1), Vertex(398 - 21), Vertex(398 + 21)],
  [Vertex(399 - 1), Vertex(399 + 1), Vertex(399 - 21), Vertex(399 + 21)],
  [Vertex(400 - 1), Vertex(400 + 1), Vertex(400 - 21), Vertex(400 + 21)],
  [Vertex(401 - 1), Vertex(401 + 1), Vertex(401 - 21), Vertex(401 + 21)],
  [Vertex(402 - 1), Vertex(402 + 1), Vertex(402 - 21), Vertex(402 + 21)],
  [Vertex(403 - 1), Vertex(403 + 1), Vertex(403 - 21), Vertex(403 + 21)],
  [Vertex(404 - 1), Vertex(404 + 1), Vertex(404 - 21), Vertex(404 + 21)],
  [Vertex(405 - 1), Vertex(405 + 1), Vertex(405 - 21), Vertex(405 + 21)],
  [Vertex(406 - 1), Vertex(406 + 1), Vertex(406 - 21), Vertex(406 + 21)],
  [Vertex(407 - 1), Vertex(407 + 1), Vertex(407 - 21), Vertex(407 + 21)],
  [Vertex(408 - 1), Vertex(408 + 1), Vertex(408 - 21), Vertex(408 + 21)],
  [Vertex(409 - 1), Vertex(409 + 1), Vertex(409 - 21), Vertex(409 + 21)],
  [Vertex(410 - 1), Vertex(410 + 1), Vertex(410 - 21), Vertex(410 + 21)],
  [Vertex(411 - 1), Vertex(411 + 1), Vertex(411 - 21), Vertex(411 + 21)],
  [Vertex(412 - 1), Vertex(412 + 1), Vertex(412 - 21), Vertex(412 + 21)],
  [Vertex(413 - 1), Vertex(413 + 1), Vertex(413 - 21), Vertex(413 + 21)],
  [Vertex(414 - 1), Vertex(414 + 1), Vertex(414 - 21), Vertex(414 + 21)],
  [Vertex(415 - 1), Vertex(415 + 1), Vertex(415 - 21), Vertex(415 + 21)],
  [Vertex(416 - 1), Vertex(416 + 1), Vertex(416 - 21), Vertex(416 + 21)],
  [Vertex(417 - 1), Vertex(417 + 1), Vertex(417 - 21), Vertex(417 + 21)],
  [Vertex(418 - 1), Vertex(418 + 1), Vertex(418 - 21), Vertex(418 + 21)],
  [Vertex(419 - 1), Vertex(419 + 1), Vertex(419 - 21), Vertex(419 + 21)],
  [Vertex(420 - 1), Vertex(420 + 1), Vertex(420 - 21), Vertex(420 + 21)],
  [Vertex(421 - 1), Vertex(421 + 1), Vertex(421 - 21), Vertex(421 + 21)],
  [Vertex(422 - 1), Vertex(422 + 1), Vertex(422 - 21), Vertex(422 + 21)],
  [Vertex(423 - 1), Vertex(423 + 1), Vertex(423 - 21), Vertex(423 + 21)],
  [Vertex(424 - 1), Vertex(424 + 1), Vertex(424 - 21), Vertex(424 + 21)],
  [Vertex(425 - 1), Vertex(425 + 1), Vertex(425 - 21), Vertex(425 + 21)],
  [Vertex(426 - 1), Vertex(426 + 1), Vertex(426 - 21), Vertex(426 + 21)],
  [Vertex(427 - 1), Vertex(427 + 1), Vertex(427 - 21), Vertex(427 + 21)],
  [Vertex(428 - 1), Vertex(428 + 1), Vertex(428 - 21), Vertex(428 + 21)],
  [Vertex(429 - 1), Vertex(429 + 1), Vertex(429 - 21), Vertex(429 + 21)],
  [Vertex(430 - 1), Vertex(430 + 1), Vertex(430 - 21), Vertex(430 + 21)],
  [Vertex(431 - 1), Vertex(431 + 1), Vertex(431 - 21), Vertex(431 + 21)],
  [Vertex(432 - 1), Vertex(432 + 1), Vertex(432 - 21), Vertex(432 + 21)],
  [Vertex(433 - 1), Vertex(433 + 1), Vertex(433 - 21), Vertex(433 + 21)],
  [Vertex(434 - 1), Vertex(434 + 1), Vertex(434 - 21), Vertex(434 + 21)],
  [Vertex(435 - 1), Vertex(435 + 1), Vertex(435 - 21), Vertex(435 + 21)],
  [Vertex(436 - 1), Vertex(436 + 1), Vertex(436 - 21), Vertex(436 + 21)],
  [Vertex(437 - 1), Vertex(437 + 1), Vertex(437 - 21), Vertex(437 + 21)],
  [Vertex(438 - 1), Vertex(438 + 1), Vertex(438 - 21), Vertex(438 + 21)],
  [Vertex(439 - 1), Vertex(439 + 1), Vertex(439 - 21), Vertex(439 + 21)],
  [Vertex(440 - 1), Vertex(440 + 1), Vertex(440 - 21), Vertex(440 + 21)]
];


impl Vertex {
  fn to_coords(self) -> (i16, i16) {
    return ((self.0 % 21) - 1, self.0 / 21 - 1);
  }

  fn as_index(self) -> usize {
    return self.0 as usize;
  }
}

impl fmt::Display for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let (x, y) = self.to_coords();
    let column_labels = "aABCDEFGHIKLMNOPORSTUu";
    try!(write!(f, "{}", column_labels.chars().nth((x + 1) as usize).unwrap()));
    return write!(f, "{}", y + 1);
  }
}
impl fmt::Debug for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    return write!(f, "{}", self);
  }
}


#[derive(Clone)]
struct String {
  color: Stone,
  num_stones: u16,

  num_pseudo_liberties: u8,
  liberty_vertex_sum: u16,
  liberty_vertex_sum_squared: u32,
}

impl String {
  fn reset(&mut self) {
    self.color = Stone::Empty;
    self.num_stones = 0;
    self.num_pseudo_liberties = 0;
    self.liberty_vertex_sum = 0;
    self.liberty_vertex_sum_squared = 0;
  }

  fn reset_border(&mut self) {
    self.color = Stone::Empty;
    self.num_stones = 0;
    self.num_pseudo_liberties = 4;
    self.liberty_vertex_sum = 32768;
    self.liberty_vertex_sum_squared = 2147483648;
  }

  fn merge(&mut self, other: &String) {
    self.num_stones += other.num_stones;
    self.num_pseudo_liberties += other.num_pseudo_liberties;
    self.liberty_vertex_sum += other.liberty_vertex_sum;
    self.liberty_vertex_sum_squared += other.liberty_vertex_sum_squared;
  }

  fn in_atari(&self) -> bool {
    return self.num_pseudo_liberties as u32 * self.liberty_vertex_sum_squared  ==
      self.liberty_vertex_sum as u32 * self.liberty_vertex_sum as u32;
  }

  fn add_liberty(&mut self, vertex: Vertex) {
    self.num_pseudo_liberties += 1;
    self.liberty_vertex_sum += vertex.0 as u16;
    self.liberty_vertex_sum_squared += vertex.0 as u32 * vertex.0 as u32;
  }

  fn remove_liberty(&mut self, vertex: Vertex) {
    self.num_pseudo_liberties -= 1;
    self.liberty_vertex_sum -= vertex.0 as u16;
    self.liberty_vertex_sum_squared -= vertex.0 as u32 * vertex.0 as u32;
  }
}

pub struct GoGame {
  size: usize,
  board: Vec<Stone>,

  vertex_hashes: Vec<u64>,
  past_position_hashes: collections::HashSet<u64>,
  position_hash: u64,

  strings: Vec<String>,
  // Head of the string for every vertex of the board.
  string_head: Vec<Vertex>,
  // Implicit representation of the linked list of all stones belonging to the
  // same String. Cyclic, indexed by Vertex.
  string_next_v: Vec<Vertex>,

  // Vector of all empty vertices.
  empty_vertices: Vec<Vertex>,
  // Position of every vertex in the vector above, to allow constant time
  // removal and addition.
  empty_v_index: Vec<usize>,

  num_black_stones: i16,

  // Precomputed direct neighbours for every vertex.
  neighbours: Vec<Vec<Vertex>>,
  diag_neighbours: Vec<Vec<Vertex>>,

  ko_vertex: Vertex,
}

impl GoGame {
  pub fn new(size: usize) -> GoGame {
    let mut rng = rand::thread_rng();

    let mut board = vec![Stone::Border; 21 * 21];
    let mut hash = 0;
    let mut vertex_hashes =  if cfg!(debug) { vec![0; 3 * board.len()] } else { vec![] };
    let mut empty_vertices = Vec::with_capacity(size * size);
    let mut empty_v_index = vec![0; 21 * 21];
    let mut neighbours = vec![vec![]; 21 * 21];
    let mut diag_neighbours = vec![vec![]; 21 * 21];
    for col in 0 .. size {
      for row in 0 .. size {
        if cfg!(debug) {
          vertex_hashes[0 * size * size + col + row * size] = rng.gen(); // Empty
          vertex_hashes[1 * size * size + col + row * size] = rng.gen(); // Black
          vertex_hashes[2 * size * size + col + row * size] = rng.gen(); // White
          // Create initial board hash.
          hash = hash ^ vertex_hashes[0 * size * size + col + row * size];
        }

        let v = GoGame::vertex(row as i16, col as i16);
        board[v.as_index()] = Stone::Empty;

        empty_v_index[v.as_index()] = empty_vertices.len();
        empty_vertices.push(v);

        neighbours[v.as_index()] = vec![Vertex(v.0 - 1), Vertex(v.0 + 1),
          Vertex(v.0 - 21), Vertex(v.0 + 21)];
        diag_neighbours[v.as_index()] = vec![Vertex(v.0 - 22), Vertex(v.0 - 20),
          Vertex(v.0 + 20), Vertex(v.0 + 22)];
      }
    }


    let mut string_head = vec![PASS; 21 * 21];
    for i in 0 .. 21 * 21 {
      string_head[i] = Vertex(i as i16);
    }

    let strings = vec![String{
      color: Stone::Empty,
      num_stones: 0,

      num_pseudo_liberties: 4,
      liberty_vertex_sum: 32768, // 2 ^ 15
      liberty_vertex_sum_squared: 2147483648, // 2 ^ 31
    }; 21 * 21];

    let past_position_hashes = if cfg!(debug) {
      collections::HashSet::with_capacity(500)
    } else {
      collections::HashSet::new()
    };
    GoGame {
      size: size,
      board: board,

      vertex_hashes: vertex_hashes,
      past_position_hashes: past_position_hashes,
      position_hash: hash,

      strings: strings,
      string_head: string_head,
      string_next_v: vec![PASS; 21 * 21],

      empty_vertices: empty_vertices,
      empty_v_index: empty_v_index,

      num_black_stones: 0,

      neighbours: neighbours,
      diag_neighbours: diag_neighbours,

      ko_vertex: PASS,
    }
  }

  pub fn reset(&mut self) {
    self.empty_vertices.clear();
    self.past_position_hashes.clear();
    self.num_black_stones = 0;
    self.ko_vertex = PASS;

    for i in 0 .. 21 * 21 {
      self.strings[i].reset_border();
      self.string_head[i] = Vertex(i as i16);
      self.string_next_v[i] = PASS;
    }

    let mut hash = 0;

    for col in 0 .. self.size {
      for row in 0 .. self.size {
        if cfg!(debug) {
          hash = hash ^ self.vertex_hashes[0 * self.size * self.size + col + row * self.size];
        }

        let v = GoGame::vertex(row as i16, col as i16);
        self.board[v.as_index()] = Stone::Empty;

        self.empty_v_index[v.as_index()] = self.empty_vertices.len();
        self.empty_vertices.push(v);
      }
    }

    self.position_hash = hash;
  }

  pub fn vertex(x: i16, y: i16) -> Vertex {
    Vertex(x + 1 + (y + 1) * 21)
  }

  fn hash_for(&self, vertex: Vertex) -> u64 {
    let offset = match self.stone_at(vertex) {
      Stone::Empty => 0,
      Stone::Black => 1,
      Stone::White => 2,
      Stone::Border => 3,
    };
    return self.vertex_hashes[offset * self.size * self.size + vertex.as_index()];
  }

  fn set_stone(&mut self, stone: Stone, vertex: Vertex) {
    let old_stone = self.board[vertex.as_index()];
    // Remove hash for old stone.
    if cfg!(debug) {
      self.position_hash = self.position_hash ^ self.hash_for(vertex);
    }
    // Place new stone and apply hash for it.
    self.board[vertex.as_index()] = stone;
    if cfg!(debug) {
      self.position_hash = self.position_hash ^ self.hash_for(vertex);
    }

    // Update empty vertex list.
    if stone == Stone::Empty {
      self.empty_v_index[vertex.as_index()] = self.empty_vertices.len();
      self.empty_vertices.push(vertex);
    } else {
      let i = self.empty_v_index[vertex.as_index()];
      {
        let last = self.empty_vertices.last().unwrap();
        self.empty_v_index[last.as_index()] = i;
      }
      self.empty_vertices.swap_remove(i);
    }

    // Update stone count for scoring.
    if old_stone == Stone::Black {
      self.num_black_stones -= 1;
    } else if stone == Stone::Black {
      self.num_black_stones += 1;
    }
  }

  pub fn play(&mut self, stone: Stone, vertex: Vertex) -> bool {
    if cfg!(debug) && !self.can_play(stone, vertex) {
      return false;
    }
    let old_num_empty_vertices = self.empty_vertices.len();
    let mut played_in_enemy_eye = true;
    for n in self.neighbours[vertex.as_index()].iter() {
      let s = self.stone_at(*n);
      if s == stone || s == Stone::Empty {
        played_in_enemy_eye = false;
      }
    }
    self.ko_vertex = PASS;

    self.join_groups_around(vertex, stone);
    self.set_stone(stone, vertex);
    self.remove_liberty_from_neighbouring_groups(vertex);
    self.capture_dead_groups(vertex, stone);

    if played_in_enemy_eye && old_num_empty_vertices == self.empty_vertices.len() {
      self.ko_vertex = *self.empty_vertices.last().unwrap();
    }

    if cfg!(debug) {
      self.check_ko();
    }
    return true;
  }

  fn place_new_stone_as_string(&mut self, vertex: Vertex, stone: Stone) {
    self.strings[vertex.as_index()].reset();
    self.strings[vertex.as_index()].num_stones += 1;

    for n in self.neighbours[vertex.as_index()].iter() {
      if self.stone_at(*n) == Stone::Empty {
        self.strings[vertex.as_index()].add_liberty(*n);
      }
    }

    self.string_head[vertex.as_index()] = vertex;
    self.string_next_v[vertex.as_index()] = vertex;
  }

  fn remove_liberty_from_neighbouring_groups(&mut self, vertex: Vertex) {
    for n in self.neighbours[vertex.as_index()].iter() {
      self.strings[self.string_head[n.as_index()].as_index()].remove_liberty(vertex);
    }
  }

  fn capture_dead_groups(&mut self, vertex: Vertex, stone: Stone) {
    for n in NEIGHBOURS[vertex.as_index()].iter() {
      if self.stone_at(*n) == stone.opponent() && self.dead(*n) {
        self.remove_group(*n);
      }
    }
  }

  fn check_ko(&mut self) {
    if self.past_position_hashes.contains(&self.position_hash) {
      println!("missed ko!");
    }
    self.past_position_hashes.insert(self.position_hash);
  }

  fn string(&self, vertex: Vertex) -> &String {
    return &self.strings[self.string_head[vertex.as_index()].as_index()];
  }

  fn num_pseudo_liberties(&self, vertex: Vertex) -> u8 {
    return self.string(vertex).num_pseudo_liberties;
  }

  fn join_groups_around(&mut self, vertex: Vertex, stone: Stone) {
    let mut largest_group_head = PASS;
    let mut largest_group_size = 0;
    for n in self.neighbours[vertex.as_index()].iter() {
      if self.stone_at(*n) == stone {
        let string = self.string(*n);
        if string.num_stones > largest_group_size {
          largest_group_size = string.num_stones;
          largest_group_head = self.string_head[n.as_index()];
        }
      }
    }

    if largest_group_size == 0 {
      self.place_new_stone_as_string(vertex, stone);
      return;
    }

    for i in 0 .. 4 {
      let n = self.neighbours[vertex.as_index()][i];
      if self.stone_at(n) == stone {
        let string_head = self.string_head[n.as_index()];
        if string_head != largest_group_head {
          // Set all the stones in the smaller string to be part of the larger
          // string.
          let mut cur = n;
          loop {
            self.string_head[cur.as_index()] = largest_group_head;
            cur = self.string_next_v[cur.as_index()];
            if cur == n {
              break;
            }
          }

          // Connect the two linked lists representing the stones in the two
          // strings.
          let tmp = self.string_next_v[largest_group_head.as_index()];
          self.string_next_v[largest_group_head.as_index()] = self.string_next_v[n.as_index()];
          self.string_next_v[n.as_index()] = tmp;

          let (small, large) = (string_head.as_index(), largest_group_head.as_index());
          if small < large {
            let (left, right) = self.strings.split_at_mut(large);
            right[0].merge(&left[small]);
          } else {
            let (left, right) = self.strings.split_at_mut(small);
            left[large].merge(&right[0]);
          }
        }
      }
    }

    self.string_next_v[vertex.as_index()] = self.string_next_v[largest_group_head.as_index()];
    self.string_next_v[largest_group_head.as_index()] = vertex;
    self.strings[largest_group_head.as_index()].num_stones += 1;
    self.string_head[vertex.as_index()] = largest_group_head;

    for n in self.neighbours[vertex.as_index()].iter() {
      if self.stone_at(*n) == Stone::Empty {
        self.strings[largest_group_head.as_index()].add_liberty(*n);
      }
    }
  }


  fn dead(&self, vertex: Vertex) -> bool {
    return self.string(vertex).num_pseudo_liberties == 0;
  }

  fn remove_group(&mut self, vertex: Vertex) {
    let mut cur = vertex;
    let string_head = self.string_head[vertex.as_index()];

    loop {
      self.set_stone(Stone::Empty, cur);
      self.string_head[cur.as_index()] = cur;
      self.strings[cur.as_index()].reset();

      for n in self.neighbours[cur.as_index()].iter() {
        let stone = self.board[n.as_index()];

        if stone == Stone::White || stone == Stone::Black {
          let neighbour_string_head = self.string_head[n.as_index()];
          if neighbour_string_head != string_head {
            self.strings[neighbour_string_head.as_index()].add_liberty(cur);
          }
        }
      }

      cur = self.string_next_v[cur.as_index()];
      if cur == vertex {
        break;
      }
    }
  }

  pub fn stone_at(&self, vertex: Vertex) -> Stone {
    return self.board[vertex.as_index()]
  }

  fn neighbours(v: Vertex) -> Vec<Vertex> {
    return vec![Vertex(v.0 - 1), Vertex(v.0 + 1), Vertex(v.0 - 21), Vertex(v.0 + 21)];
  }

  pub fn can_play(&self, stone: Stone, vertex: Vertex) -> bool {
    // Can't play if the vertex is not empty or would be ko.
    if self.stone_at(vertex) != Stone::Empty || vertex == self.ko_vertex {
      return false;
    }

    // Can definitely play if the placed stone will have at least one direct
    // freedom (can't be ko).
    for n in self.neighbours[vertex.as_index()].iter() {
      if self.stone_at(*n) == Stone::Empty {
        return true;
      }
    }

    // For all checks below, the newly placed stone is completely surrounded by
    // enemy and friendly stones.

    // Don't allow to destroy eye-like points.
    let mut surrounded_by_own = true;
    let opponent = stone.opponent();
    for n in self.neighbours[vertex.as_index()].iter() {
      let s = self.stone_at(*n);
      if s == opponent || s == Stone::Empty {
        surrounded_by_own = false;
        break;
      }
    }
    if surrounded_by_own {
      let mut enemy_count = 0;
      let mut border = 0;
      for n in self.diag_neighbours[vertex.as_index()].iter() {
        let s = self.stone_at(*n);
        if s == opponent {
          enemy_count += 1;
        } else if s == Stone::Border {
          border = 1;
        }
      }

      if enemy_count + border < 2 {
        // eye-like point
        return false;
      }
    }

    // Allow to play if the placed stones connects to a group that still has at
    // least one other liberty after connecting.
    for n in self.neighbours[vertex.as_index()].iter() {
      if self.stone_at(*n) == stone && !self.string(*n).in_atari() {
        return true;
      }
    }

    // Allow to play if the placed stone will kill at least one group.
    for n in self.neighbours[vertex.as_index()].iter() {
      if self.stone_at(*n) == stone.opponent() && self.string(*n).in_atari() {
        return true;
      }
    }

    // Don't allow to play if the stone would be dead or kill its own group.
    return false;
  }

  pub fn random_move(&self, stone: Stone, rng: &mut rand::StdRng) -> Vertex {
    let num_empty = self.empty_vertices.len();
    let start_vertex = rng.gen_range(0, num_empty);
    let mut i = start_vertex;

    loop {
      let v = self.empty_vertices[i];
      if self.can_play(stone, v) {
        return v;
      }
      i += 1;
      if i == num_empty {
        i = 0;
      }
      if i == start_vertex {
        return PASS;
      }
    }
  }

  pub fn possible_moves(&mut self, stone: Stone) -> Vec<Vertex> {
    return self.empty_vertices.iter().map(|v| v.clone())
      .filter(|v| self.can_play(stone, *v)).collect::<Vec<_>>();
  }

  pub fn chinese_score(&self) -> i16 {
    let num_white_stones = (self.size * self.size) as i16 - self.num_black_stones - self.empty_vertices.len() as i16;

    let mut eye_score = 0;
    for v in self.empty_vertices.iter() {
      let mut num_black = 0;
      let mut num_white = 0;

      for n in self.neighbours[v.as_index()].iter() {
        let s = self.stone_at(*n);
        if s == Stone::Black {
          num_black += 1;
        } else if s == Stone::White {
          num_white += 1;
        } else {
          num_black += 1;
          num_white += 1;
        }
      }

      if num_black == 4 {
        eye_score += 1;
      } else if num_white == 4 {
        eye_score -= 1;
      }
    }

    return self.num_black_stones - num_white_stones + eye_score;
  }
}

impl fmt::Display for GoGame {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let column_labels = "ABCDEFGHIKLMNOPORSTU";
    try!(write!(f, "\x1b[0;37m    "));
    for col in 0 .. self.size {
      try!(write!(f, " {}", column_labels.chars().nth(col).unwrap()));
    }
    try!(write!(f, "\n"));

    for row in 0 .. self.size {
      try!(write!(f, " {:2} \x1b[43m\x1b[1;37m ", row + 1));
      for col in 0 .. self.size {
        try!(match self.stone_at(GoGame::vertex(col as i16, row as i16)) {
          Stone::Black => write!(f, "\x1b[30m\u{25CF}\x1b[37m "),
          Stone::White => write!(f, "\u{25CF} "),
          _ => write!(f, "\u{00b7} ")
        });
      }
      try!(write!(f, "\x1b[0;37m {:2}\n", row + 1));
    }

    try!(write!(f, "    "));
    for col in 0 .. self.size {
      try!(write!(f, " {}", column_labels.chars().nth(col).unwrap()));
    }

    return write!(f, "");
  }
}

#[cfg(test)]
mod test;
