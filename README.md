# RS-232 Serial Port Reader

A rust app made to read any RS - 232 data specifically for Arduino and sensors connected to them.

## How to use


### Code Side
The buttons say what they do

### Arduino Side
Can be more tricky because Arduinos don't like to behave use `Serial.write((byte)&var,4)` to write var so the code can read it


## How the code works
 
### Port handling 
the code uses the "serialport" crate to manage physical ports but also has its own port trait in order to be able to "split" ports meaning that the code will take each value and send it to a different graph based on number of splits to read multiple different sensors from one port. Ports are opened when pressing the "open port" button to make sure they can be opened and then splitting them if needed before trying to make a graph.
### Data handling
the data is saved very natively while the code is running, just saving the bytes read from the port for each open graph. The code implements 2 ways of saving the data for after is stops running: raw bytes, and CSV. Raw bytes has the advantage of being reopenable by the code later, and CSV by anything else.
### UI
the "Iced" crate is used to make a UI. Their docs put it better then I can [link](https://iced.rs/).
