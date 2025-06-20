use actix_web::{web, App, HttpServer, HttpResponse, Result};
use actix_files::Files;
use serde::{Deserialize, Serialize};
use crate::{GameBoard, Direction};

#[derive(Serialize, Deserialize)]
pub struct GameState {
    board: [[u32; 4]; 4],
    score: u32,
    max_tile: u32,
    game_over: bool,
}

#[derive(Deserialize)]
pub struct MoveRequest {
    direction: String,
}

pub async fn start_server() -> std::io::Result<()> {
    println!("Starting web server at http://localhost:8080");
    
    HttpServer::new(|| {
        App::new()
            .service(Files::new("/static", "static").show_files_listing())
            .route("/", web::get().to(index))
            .route("/api/new-game", web::post().to(new_game))
            .route("/api/move", web::post().to(make_move))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn index() -> Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("static/index.html")?)
}

async fn new_game() -> Result<HttpResponse> {
    let game = GameBoard::new();
    let game_state = GameState {
        board: game.get_board(),
        score: game.get_score(),
        max_tile: game.get_max_tile(),
        game_over: game.is_game_over(),
    };
    
    Ok(HttpResponse::Ok().json(game_state))
}

async fn make_move(req: web::Json<MoveRequest>) -> Result<HttpResponse> {
    let mut game = GameBoard::new(); // In a real app, you'd store game state
    
    let direction = match req.direction.as_str() {
        "up" => Direction::Up,
        "down" => Direction::Down,
        "left" => Direction::Left,
        "right" => Direction::Right,
        _ => return Ok(HttpResponse::BadRequest().body("Invalid direction")),
    };
    
    if game.move_tiles(direction) {
        game.add_random_tile_self();
    }
    
    let game_state = GameState {
        board: game.get_board(),
        score: game.get_score(),
        max_tile: game.get_max_tile(),
        game_over: game.is_game_over(),
    };
    
    Ok(HttpResponse::Ok().json(game_state))
} 