# Rusty Pong 🎮

A classic Pong implementation built with the Bevy game engine and Rapier physics. Created as a holiday learning project
during Christmas 2024, focusing on creating a complete game loop in the most rusty way possible.

## Game Overview

Two paddles, one ball, endless possibilities. This minimalist take on Pong emphasizes smooth physics-based gameplay with
a clean aesthetic. The AI opponent provides a challenging experience by predicting ball trajectories in real-time.

Looking through the lens of Essential Experience:

- The core experience is that moment-to-moment tension of anticipating where the ball will go and positioning your
  paddle just right
- Clean visuals and physics-driven ball movement keep the focus purely on that core gameplay tension

Through the lens of Fun:

- The satisfaction of perfectly timing a return
- The mounting excitement as volleys get longer
- The strategic depth of choosing shot angles and positioning

## Features

- Smooth, physics-based gameplay using Rapier2D
- AI opponent with predictive ball tracking
- Modern scoring system with deuce handling
- Pause functionality
- Background music toggle (M key)
- Clean state management flow:
  - Splash screen
  - Active gameplay
  - Pause menu
  - Victory/defeat screen

## Controls

- Player movement: W/S or Up/Down arrow keys
- Pause: Space
- Music toggle: M
- Start new game: Space (from victory/defeat screen)

## Technical Stack

Built with:

- Bevy 0.15.0 - A data-driven game engine
- Rapier2D - Modern physics engine providing realistic ball movement and collisions
- Efficient state management for clean game flow
- Component-based architecture following Bevy best practices

## Build and Run

Make sure you have Rust installed, then:

```bash
# Clone the repository
git clone https://github.com/yourusername/rusty-pong.git
cd rusty-pong

# Run in debug mode
cargo run

# Or build and run in release mode for better performance
cargo run --release
```

## Design Philosophy

Through the lens of Project Focus:

- Built in a few concentrated hours
- Emphasis on completing a polished game loop rather than feature creep
- Minimal but complete - every element serves the core experience

Through the lens of Elegance:

- Clean separation of concerns in the codebase
- Focused gameplay mechanics that work together harmoniously
- No unnecessary complexity - every feature supports the core game loop

## Development Roadmap

This project was created as a learning exercise to understand:

- Game development with Bevy
- Physics integration with Rapier
- Game state management
- Audio handling in games
- AI behavior implementation
- The state of WASM game development with Rust

## License

MIT - See LICENSE file for details.

## Acknowledgments

- The Bevy community for their excellent documentation and examples
- Classic Pong for providing timeless gameplay inspiration
