"""
Utility to generate a placeholder Charlie Kirk image.
This should be replaced with an actual image of Charlie Kirk.
"""
from PIL import Image, ImageDraw, ImageFont
import os

def create_placeholder_image(output_path, width=800, height=600):
    """Create a placeholder image for Charlie Kirk."""
    img = Image.new('RGB', (width, height), color='#FF6B6B')
    draw = ImageDraw.Draw(img)
    
    # Draw text
    text = "CHARLIE KIRK\nFOCUS!"
    
    # Try to use a larger font, fall back to default if not available
    try:
        font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 80)
    except:
        font = ImageFont.load_default()
    
    # Get text bounding box
    bbox = draw.textbbox((0, 0), text, font=font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]
    
    # Center text
    x = (width - text_width) // 2
    y = (height - text_height) // 2
    
    draw.text((x, y), text, fill='white', font=font, align='center')
    
    # Save image
    img.save(output_path)
    print(f"Created placeholder image at {output_path}")

if __name__ == "__main__":
    # Create assets directory if it doesn't exist
    os.makedirs("assets/images", exist_ok=True)
    
    # Create placeholder image
    create_placeholder_image("assets/images/charlie_kirk.png")
