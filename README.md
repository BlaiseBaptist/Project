# RS-232 Serial Port Reader

A rust app made to read any RS - 232 data specifically for Arduino and sensors connected to them.

## How to use

its easy just do

### Code Side
    
idk just like press the buttons 

### Arduino Side
    
can be more tricky because arduinos dont like to behave use `Serial.write((byte)&var,4)` to write var so the code can read it

## How the code works
 
### Port handling 
the code uses the "serialport" crate to manange phyical ports but also has its own port trait inorder to be able to "split" ports meaning that the code will take each value and send it to a differnt graph based on numbre of splits to read multipule different sensors from one port. ports are opened when pressing the "open port" button to make sure they can be opened and then spliting them if needed before trying to make a graph.
### Data handling
the data is saved very natively while the code is running, just saving the bytes read from the port for each open graph. the code implemnts 2 ways of saving the data for after is stops running: raw bytes, and CSV. raw bytes has the advantage of being reopenable by the code later, and csv by anything else.
### UI
the "Iced" crate is used to make a UI. it puts it better then me [link](https://iced.rs/).
