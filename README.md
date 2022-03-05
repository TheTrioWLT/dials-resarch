# dials-research
Simulation software used by human factors resarchers at Embry Riddle Aeronautical University

## Requirements:

### Tracking Task 
- We want a compensatory tracking task whereby the user's goal is to keep the ball as close as possible to the cross in the middle of the screen using a joystick. 
- The ball will try to escape the cross along the following parameters: 
  - X - ball will travel in a random direction away from the cross 
  - Y - ball will travel at a random speed 
  - Z - ball will randomly change directions after a random amount of time 
  - These parameters will pull from a CSV file which can be edited by the researchers. 
- The ball should never exit the box. 
- Output should be in RMSE (Root Mean Square Error) which measures how far away the ball is from the cross at 60hz, then averaged for each participant. 
- Output should be presented as a CSV text file 

### Gauges 
- At the bottom of the tracking task, there should be a space for gauges 
- The researchers should be able to specify the number of gauges present, including 0 gauges 
- Gauges should have the following design (see example image below): 
  - 10 small white ticks spaced equidistantly around the outside of an imaginary circle. 
  - A yellow bar that “fills” the outside of the gauge denoting units of 1,000, 2,000, and so forth.  
  - A yellow needle that rotates around the inside of the gauge denoting units of 100, 200, et cetera.  
- Researchers must be able to specify the “in-range zone”/“out-of-range zone” for each gauge 
- Researchers must be able to specify the audio file related to each gauge 

### Audio task 
- We need to be able to have several audio files play at specific times during the tracking task when gauges drift out of range 
  - Each audio file is associated with one of the gauges present 
- We must be able to select the audio files we want to play 
- We must be able to specify the times at which the audio files are played during the task and which gauge they are related to 
- We need to be able to specify these parameters in the CSV file. 
- The participant will press a corresponding key on the keyboard when they hear each sound 
  - We must be able to specify/select the “correct identification” key corresponding to each audio file 
- Output should include both accuracy and response times for each response. 
  - Response times should be in milliseconds starting when the audio file begins playing 
