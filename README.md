# DTS - Dial Tracking System

The goal of this simulation software is to study users'
reaction time in the presence of distractions. The distraction in
question is a ball located at the center of the window, within a 
designated box, which the user has to control at all times. 
This ball will move at random velocities and times, 
with parameters that can be specified.

The reaction time test involves a set of dials located at the
bottom of the window. These dials will trigger an alarm sound
or other specified sound, prompting the user to press the 
corresponding key attached to the dial. The program will then
track the position of the ball in relation to the center of the
screen, which is represented by a cross, and output test values
such as the Root Mean Square Error (RMSE) in a .csv file.


>The program was made for the use of the human factor's team 
at Embry-Riddle Aeronautical University to help with their
research. However, the program may be used by any audience.


**More description and guides are shown in this document.**


## User Interface System
This section shows how the UI of the program looks like. 

![UI system](../dials-research/extras/ui_system.png)

### Tracking Frame 
The main interaction of the program is in the white outlined
box, or Tracking Frame. Within this box there is a cross
marking its center. The green dot represents the ball, 
and it will move freely, unless controlled, throughout 
the tracking frame never going off limits.




## Authors
[@Luke Newcomb](https://github.com/newcomb-luke)

[@Walter Hernandez](https://github.com/HernanW4)

[@Troy Neubauer](https://github.com/TroyNeubauer)


