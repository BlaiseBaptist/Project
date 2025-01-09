# RS-232 Serial Port Reader

A rust app made to read any RS - 232 data specifically for Arduino and sensors connected to them.

## How to Use

Send stream of u32s to a serial port and select it with the app

## Todo list
1. refactor 
    * graph mod in specific
1. improve UI
   * check how the scaling and panning feel
   * fix x shifting and scaling interaction to make it easier to see the end of it
1. go from time based update to waiting based updates for the graph and move that to the port
1. figure out how to deal with different types of data from ports
1. add keyboard shortcuts
1. reduce memory usage of stored data
