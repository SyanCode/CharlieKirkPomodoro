#!/usr/bin/env python3
"""
Charlie Kirk Pomodoro - A focus monitoring tool
Monitors user attention via camera and eye tracking, displays Charlie Kirk images when distracted.
"""

import cv2
import mediapipe as mp
import numpy as np
import pygame
import time
import os
import sys
from pathlib import Path

class CharlieKirkPomodoro:
    def __init__(self, config=None):
        """Initialize the Charlie Kirk Pomodoro system."""
        self.config = config or {}
        
        # Default configuration
        self.distraction_threshold = self.config.get('distraction_threshold', 3.0)  # seconds
        self.gaze_away_threshold = self.config.get('gaze_away_threshold', 0.15)  # normalized units
        
        # Initialize MediaPipe Face Mesh
        self.mp_face_mesh = mp.solutions.face_mesh
        self.face_mesh = self.mp_face_mesh.FaceMesh(
            max_num_faces=1,
            refine_landmarks=True,
            min_detection_confidence=0.5,
            min_tracking_confidence=0.5
        )
        
        # Initialize camera
        self.camera = None
        self.camera_index = self.config.get('camera_index', 0)
        
        # Distraction tracking
        self.distracted_start_time = None
        self.is_distracted = False
        
        # Assets paths
        self.base_path = Path(__file__).parent.parent
        self.image_path = self.base_path / "assets" / "images" / "charlie_kirk.png"
        self.sound_path = self.base_path / "assets" / "sounds" / "alert.wav"
        
        # Pygame for sound and image display
        pygame.init()
        pygame.mixer.init()
        
        # Load assets
        self.load_assets()
        
        # Display window
        self.alert_window = None
        
    def load_assets(self):
        """Load Charlie Kirk image and alert sound."""
        if not self.image_path.exists():
            print(f"Warning: Image not found at {self.image_path}")
            self.charlie_image = None
        else:
            self.charlie_image = pygame.image.load(str(self.image_path))
            
        if not self.sound_path.exists():
            print(f"Warning: Sound not found at {self.sound_path}")
            self.alert_sound = None
        else:
            self.alert_sound = pygame.mixer.Sound(str(self.sound_path))
    
    def initialize_camera(self):
        """Initialize the camera."""
        self.camera = cv2.VideoCapture(self.camera_index)
        if not self.camera.isOpened():
            raise RuntimeError(f"Could not open camera at index {self.camera_index}")
        return True
    
    def calculate_gaze_direction(self, face_landmarks, frame_width, frame_height):
        """
        Calculate if user is looking at the screen based on eye landmarks.
        Returns True if user is looking away (distracted).
        """
        # Get eye landmarks (iris centers)
        # Left eye iris center: 468-473
        # Right eye iris center: 473-478
        
        # Get left and right eye centers
        left_eye_indices = [33, 133, 160, 159, 158, 144, 145, 153]  # Left eye landmarks
        right_eye_indices = [362, 263, 387, 386, 385, 373, 374, 380]  # Right eye landmarks
        
        left_eye_points = []
        right_eye_points = []
        
        for idx in left_eye_indices:
            landmark = face_landmarks.landmark[idx]
            left_eye_points.append([landmark.x, landmark.y])
            
        for idx in right_eye_indices:
            landmark = face_landmarks.landmark[idx]
            right_eye_points.append([landmark.x, landmark.y])
        
        # Calculate eye centers
        left_eye_center = np.mean(left_eye_points, axis=0)
        right_eye_center = np.mean(right_eye_points, axis=0)
        
        # Calculate face center (nose tip)
        nose_tip = face_landmarks.landmark[1]
        face_center = np.array([nose_tip.x, nose_tip.y])
        
        # Calculate deviation from center
        eye_center = (left_eye_center + right_eye_center) / 2
        deviation = np.linalg.norm(eye_center - face_center)
        
        # If deviation is large, user is looking away
        return deviation > self.gaze_away_threshold
    
    def process_frame(self, frame):
        """
        Process a camera frame to detect face and gaze direction.
        Returns True if user is detected and looking at screen, False otherwise.
        """
        # Convert to RGB for MediaPipe
        rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
        results = self.face_mesh.process(rgb_frame)
        
        if not results.multi_face_landmarks:
            # No face detected
            return False
        
        # Get first face
        face_landmarks = results.multi_face_landmarks[0]
        
        # Check gaze direction
        height, width, _ = frame.shape
        looking_away = self.calculate_gaze_direction(face_landmarks, width, height)
        
        return not looking_away  # Return True if looking at screen
    
    def show_charlie_kirk_alert(self):
        """Display Charlie Kirk image in fullscreen and play alert sound."""
        if not self.alert_window:
            # Create fullscreen window
            screen_info = pygame.display.Info()
            self.alert_window = pygame.display.set_mode(
                (screen_info.current_w, screen_info.current_h),
                pygame.FULLSCREEN
            )
            pygame.display.set_caption("FOCUS!")
        
        # Display Charlie Kirk image
        if self.charlie_image:
            # Scale image to fit screen
            screen_size = self.alert_window.get_size()
            image_size = self.charlie_image.get_size()
            
            # Calculate scaling to fit screen
            scale_x = screen_size[0] / image_size[0]
            scale_y = screen_size[1] / image_size[1]
            scale = min(scale_x, scale_y)
            
            new_size = (int(image_size[0] * scale), int(image_size[1] * scale))
            scaled_image = pygame.transform.scale(self.charlie_image, new_size)
            
            # Center image
            x = (screen_size[0] - new_size[0]) // 2
            y = (screen_size[1] - new_size[1]) // 2
            
            self.alert_window.fill((255, 0, 0))  # Red background
            self.alert_window.blit(scaled_image, (x, y))
            pygame.display.flip()
        
        # Play alert sound (loop)
        if self.alert_sound and not pygame.mixer.get_busy():
            self.alert_sound.play(-1)  # Loop indefinitely
    
    def hide_charlie_kirk_alert(self):
        """Hide Charlie Kirk alert and stop sound."""
        if self.alert_window:
            pygame.display.quit()
            self.alert_window = None
            pygame.display.init()
        
        # Stop sound
        if self.alert_sound:
            pygame.mixer.stop()
    
    def run(self):
        """Main monitoring loop."""
        print("Charlie Kirk Pomodoro starting...")
        print("Press 'q' to quit")
        
        if not self.initialize_camera():
            print("Failed to initialize camera")
            return
        
        try:
            while True:
                # Read frame from camera
                ret, frame = self.camera.read()
                if not ret:
                    print("Failed to read from camera")
                    break
                
                # Process frame
                is_focused = self.process_frame(frame)
                
                current_time = time.time()
                
                if not is_focused:
                    # User is not focused
                    if self.distracted_start_time is None:
                        self.distracted_start_time = current_time
                    
                    # Check if distracted for long enough
                    distraction_duration = current_time - self.distracted_start_time
                    if distraction_duration >= self.distraction_threshold and not self.is_distracted:
                        print("User is distracted! Showing alert...")
                        self.is_distracted = True
                        self.show_charlie_kirk_alert()
                else:
                    # User is focused
                    if self.is_distracted:
                        print("User is back! Hiding alert...")
                        self.hide_charlie_kirk_alert()
                        self.is_distracted = False
                    
                    self.distracted_start_time = None
                
                # Display monitoring view (small window)
                if not self.is_distracted:
                    small_frame = cv2.resize(frame, (320, 240))
                    status = "FOCUSED" if is_focused else "CHECKING..."
                    color = (0, 255, 0) if is_focused else (0, 255, 255)
                    cv2.putText(small_frame, status, (10, 30), 
                               cv2.FONT_HERSHEY_SIMPLEX, 0.7, color, 2)
                    cv2.imshow("Charlie Kirk Pomodoro", small_frame)
                
                # Check for pygame events
                for event in pygame.event.get():
                    if event.type == pygame.QUIT:
                        return
                    if event.type == pygame.KEYDOWN:
                        if event.key == pygame.K_ESCAPE or event.key == pygame.K_q:
                            return
                
                # Check for OpenCV window events
                key = cv2.waitKey(1) & 0xFF
                if key == ord('q'):
                    break
                    
        finally:
            print("Shutting down...")
            self.cleanup()
    
    def cleanup(self):
        """Clean up resources."""
        if self.camera:
            self.camera.release()
        cv2.destroyAllWindows()
        self.hide_charlie_kirk_alert()
        pygame.quit()

def main():
    """Main entry point."""
    try:
        pomodoro = CharlieKirkPomodoro()
        pomodoro.run()
    except KeyboardInterrupt:
        print("\nInterrupted by user")
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        return 1
    return 0

if __name__ == "__main__":
    sys.exit(main())
