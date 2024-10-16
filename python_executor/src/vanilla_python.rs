use std::process::Command;

fn run_python_script(script: &str) -> Result<String, std::io::Error> {
    let child = Command::new("python")
        .arg("-c")
        .arg(script)
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_python_script() {
        let script = r#"
from chess import BLACK, SQUARE_NAMES, WHITE, Board, Move
from chess import piece_name as get_piece_name
from typing import Literal, Annotated


def validate_turn(board: Board, player: Literal["white", "black"]) -> None:
    """Validate that it is the player's turn to move."""
    last_move = board.peek() if board.move_stack else None
    if last_move is not None:
        if player == "white" and board.color_at(last_move.to_square) == WHITE:
            raise ValueError("It is not your turn to move. Wait for black to move.")
        if player == "black" and board.color_at(last_move.to_square) == BLACK:
            raise ValueError("It is not your turn to move. Wait for white to move.")
    elif last_move is None and player != "white":
        raise ValueError("It is not your turn to move. Wait for white to move first.")


def get_legal_moves(
    board: Board, player: Literal["white", "black"]
) -> Annotated[str, "A list of legal moves in UCI format."]:
    """Get legal moves for the given player."""
    validate_turn(board, player)
    legal_moves = list(board.legal_moves)
    if player == "black":
        legal_moves = [
            move for move in legal_moves if board.color_at(move.from_square) == BLACK
        ]
    elif player == "white":
        legal_moves = [
            move for move in legal_moves if board.color_at(move.from_square) == WHITE
        ]
    else:
        raise ValueError("Invalid player, must be either 'black' or 'white'.")
    if not legal_moves:
        return "No legal moves. The game is over."

    return "Possible moves are: " + ", ".join([move.uci() for move in legal_moves])


def get_board(board: Board) -> str:
    return str(board)


def make_move(
    board: Board,
    player: Literal["white", "black"],
    thinking: Annotated[str, "Thinking for the move."],
    move: Annotated[str, "A move in UCI format."],
) -> Annotated[str, "Result of the move."]:
    """Make a move on the board."""
    validate_turn(board, player)
    newMove = Move.from_uci(move)
    board.push(newMove)

    # Print the move.
    print("-" * 50)
    print("Player:", player)
    print("Move:", newMove.uci())
    print("Thinking:", thinking)
    print("Board:")
    print(board.unicode(borders=True))

    # Get the piece name.
    piece = board.piece_at(newMove.to_square)
    assert piece is not None
    piece_symbol = piece.unicode_symbol()
    piece_name = get_piece_name(piece.piece_type)
    if piece_symbol.isupper():
        piece_name = piece_name.capitalize()
    return f"Moved {piece_name} ({piece_symbol}) from {SQUARE_NAMES[newMove.from_square]} to {SQUARE_NAMES[newMove.to_square]}."


async def chess_game():  # type: ignore
    board = Board()

    def get_legal_moves_black() -> str:
        return get_legal_moves(board, "black")

    def get_legal_moves_white() -> str:
        return get_legal_moves(board, "white")

    def make_move_black(
        thinking: Annotated[str, "Thinking for the move"],
        move: Annotated[str, "A move in UCI format"],
    ) -> str:
        return make_move(board, "black", thinking, move)

    def make_move_white(
        thinking: Annotated[str, "Thinking for the move"],
        move: Annotated[str, "A move in UCI format"],
    ) -> str:
        return make_move(board, "white", thinking, move)

    def get_board_text() -> Annotated[str, "The current board state"]:
        return get_board(board)


board = Board()


def get_legal_moves_black() -> str:
    res = get_legal_moves(board, "white")
    print(res)
    return res


get_legal_moves_black()
"#;

        match run_python_script(script) {
            Ok(output) => println!("Output: {}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
