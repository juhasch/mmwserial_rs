#!/usr/bin/env python3
"""Test script for mmwserial radar reader."""

import time
import logging
from mmwserial import RadarReader

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def main():
    """Main test function."""
    # Create reader instance
    port = "/dev/ttyUSB1"  # Adjust to your port
    logger.info(f"Opening port {port}")
    
    try:
        reader = RadarReader(port)
        logger.info("Successfully created reader")
        
        last_frame = None
        missed_frames = 0
        total_frames = 0
        
        # Read packets for a few seconds
        start_time = time.time()
        while time.time() - start_time < 10:  # Run for 10 seconds
            if packet := reader.read_packet():
                total_frames += 1
                frame = packet.header.frame_number
                
                if last_frame is not None and frame != last_frame + 1:
                    missed = frame - last_frame - 1
                    missed_frames += missed
                    logger.warning(f"Missed {missed} frames between {last_frame} and {frame}")
                
                last_frame = frame
                logger.info(f"Frame {frame}: {packet.header.num_detected_obj} objects, "
                          f"{packet.header.total_packet_len} bytes")
            else:
                logger.warning("No packet received")
        
        # Print statistics
        duration = time.time() - start_time
        logger.info(f"\nStatistics over {duration:.1f} seconds:")
        logger.info(f"Total frames: {total_frames}")
        logger.info(f"Missed frames: {missed_frames}")
        logger.info(f"Frame rate: {total_frames/duration:.1f} fps")
        logger.info(f"Frame loss: {100*missed_frames/(total_frames+missed_frames):.1f}%")
            
    except Exception as e:
        logger.error(f"Error: {e}")

if __name__ == "__main__":
    main() 