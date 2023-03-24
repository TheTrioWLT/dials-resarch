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

### Ball Movement
The Ball on the screen, the little green dot, will move freely
around the Tracking Frame. It will bounce back every time 
it hits the walls. Its direction will change randomly at 
given intervals. This intervals can be customized in the 
[config](#config-setup) file. A user may control the ball 
in 2 ways. One is by using the arrow keys, and the most 
preferred one is the joystick input where the user may 
use a joystick. **BEWARE!!** Not all joysticks have been 
tested, we cannot guarantee that all joysticks can work.


### Dials 
Below the Tracking Frame, dials are setup. One is able to 
configure the numbers of dials that appear in the program
via the [config](#config-setup) file. Each dial represents a trial
where an alarm should go off. The alarm will go off once its dial
has reached the time limit. It is possible to change alarm sounds
based on the dial. Once an alarm goes off, the dial will wait for a key input
and until then, its needle will remain still outside the dial's bound. The key input
can be any key pressed after the alarm goes off, no matter if it's the wrong or right
key. The right key has to be specified in the config file as well or default values
will be used. 


## Config Setup
This section aims to help understand how to properly use 
the config file used in the program. Is important to know
that syntax and typos are important in this config as the 
extension of the file is **toml**. I will not explain 
how toml works, I recommend visiting [toml_template](https://toml.io/en/)
for any questions on why something is the way it is. 

>A config file will be automatically made if no config file 
is found when the program is runned.

Make sure to have your config file with named as such **config_file.toml**


### Input Mode
This is how one specifies the type of input the program should use.
It will **NOT** detect it automatically.

Input Mode should be the first line of the config file, and suggest
leaving it like that to not have any issues. 

If the config file is made by the program, it will then 
choose keyboard as the default input mode. Something 
like this
```toml
input_mode = "keyboard"
```


To change the input mode to joystick it will be very similar
to keyboard. 

```toml
input_mode = "joystick"
```


## Authors
[@Luke Newcomb](https://github.com/newcomb-luke)

[@Walter Hernandez](https://github.com/HernanW4)

[@Troy Neubauer](https://github.com/TroyNeubauer)


