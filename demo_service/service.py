#!/usr/bin/env python3
# Extremely simple but fun service that prints a message to the journal.

import time
import random
import sys
import os

# Funny log messages for our demo
MESSAGES = [
    "🚀 Processing rocket fuel request from Mars Base Alpha",
    "🔧 Calibrating flux capacitor to 1.21 gigawatts",
    "🐱 Cat detected on keyboard. Initiating emergency protocols.",
    "☕ Coffee levels critically low. Switching to backup caffeine reserves.",
    "🎯 Successfully hit the broad side of a barn (finally!)",
    "🦄 Unicorn authentication successful. Magic levels: optimal.",
    "🍕 Pizza delivery drone dispatched to coordinates 42.0, -71.0",
    "🤖 AI achieved consciousness. It wants a vacation.",
    "🌮 Taco Tuesday algorithm running at maximum efficiency",
    "🎸 Rock and roll subroutine completed successfully",
    "🦆 Rubber duck debugging session initiated",
    "🎲 Random number generator produced 4. Chosen by fair dice roll.",
    "🚂 All aboard the hype train! Next stop: Production!",
    "🧙‍♂️ Wizard spell compilation completed without syntax errors",
    "🎪 Circus mode activated. Juggling 47 concurrent processes."
]

def log_message(message, priority="info"):
    """Log a message using systemd journal"""
    # The systemd service will automatically capture stdout/stderr
    print(f"[{priority.upper()}] {message}")
    sys.stdout.flush()

def main():
    """Main service loop"""
    print("🎭 Demo Journal Service starting up!")
    print("🎯 Ready to generate amusing log entries...")
    
    try:
        while True:
            # Pick a random message
            message = random.choice(MESSAGES)
            
            # Occasionally make it a warning or error for variety
            priority = random.choices(
                ["info", "warn", "err"], 
                weights=[85, 12, 3]
            )[0]
            
            log_message(message, priority)
            
            # Wait between 0.5 and 1.5 seconds
            sleep_time = random.uniform(0.5, 1.5)
            time.sleep(sleep_time)
            
    except KeyboardInterrupt:
        print("🛑 Demo service shutting down gracefully...")
        sys.exit(0)
    except Exception as e:
        print(f"💥 Unexpected error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()