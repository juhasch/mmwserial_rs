#!/usr/bin/env python3
"""Test script for mmwserial UDP reader."""

import time
import logging
import socket
from mmwserial import UDPReader

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def main():
    """Main test function."""
    # Create UDP socket for sending test data
    interface = "0.0.0.0"
    port = 8080
    frame_size = 1024
    
    # Create sender socket
    sender = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    
    # Create reader instance
    logger.info(f"Opening UDP reader on {interface}:{port}")
    
    try:
        reader = UDPReader(interface, port, frame_size, timeout_ms=1000)
        logger.info("Successfully created reader")
        
        # Send some test frames
        num_frames = 5
        logger.info(f"Sending {num_frames} test frames")
        
        for i in range(num_frames):
            # Create test frame with frame number
            frame = bytearray(frame_size)
            frame[0] = i  # First byte is frame number
            
            # Send frame
            sender.sendto(frame, (interface, port))
            logger.info(f"Sent frame {i}")
            time.sleep(0.1)  # Small delay between frames
        
        # Read frames
        logger.info("Reading frames...")
        frames = reader.read_frames(num_frames)
        
        # Print results
        logger.info(f"\nReceived {len(frames)} frames:")
        for i, frame in enumerate(frames):
            logger.info(f"Frame {i}: first byte = {frame[0]}")
            
    except Exception as e:
        logger.error(f"Error: {e}")
    finally:
        sender.close()

if __name__ == "__main__":
    main() 