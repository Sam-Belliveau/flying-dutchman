use std::str::FromStr;

use chess::ChessMove;
use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum UCIToken {
    #[token("fd_test")]
    FlyingDutchmanTest,

    #[token("uci")]
    UCI,

    #[token("ucinewgame")]
    NewGame,

    #[token("isready")]
    IsReady,

    #[token("position")]
    Position,

    #[token("setoption")]
    SetOption,

    #[token("go")]
    Go,

    #[token("stop")]
    Stop,

    #[token("quit")]
    Quit,
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum UCIGoToken {
    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    Number(u64),

    #[regex(r"[a-h][1-8][a-h][1-8]+", |lex| ChessMove::from_str(lex.slice()).ok())]
    Move(ChessMove),

    #[token("searchmoves")]
    SearchMoves,

    #[token("ponder")]
    Ponder,

    #[token("wtime")]
    WhiteTime,

    #[token("btime")]
    BlackTime,

    #[token("winc")]
    WhiteTimeInc,

    #[token("binc")]
    BlackTimeInc,

    #[token("movestogo")]
    MovesToGo,

    #[token("depth")]
    Depth,

    #[token("nodes")]
    Nodes,

    #[token("mate")]
    Mate,

    #[token("movetime")]
    MoveTime,

    #[token("infinite")]
    Infinite,
}
