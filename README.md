# Mazinlab Third Generation (Gen3) MKID Readout Interactive GUI
This project repo is under active development. Check this page and ```git pull``` often for updates

It is directly connected to the ```gen3_rpc``` repo, which hosts the server connection to the Xilinx RFSoC


See also: https://github.com/MazinLab/MKIDGen3.git

This repo yields an interactive gui for controlling the Gen3 readout, including logging and status updates. 
#### Please comment all new code or changes before committing

## Download the reading-rainbow Repo

```
git clone git@github.com:MazinLab/readout-gui.git
```
## Download the gen3_rpc Repo 
```
git clone git@github.com:MazinLab/gen3_rpc.git
```

## Basic Structure 

1. ```gui.rs``` creates the gui. It defines a Struct for each pane in the gui and imports associated files that control each pane as crates.
2. ```worker.rs``` is a backend communication between then Gen3Board (in gen3_rpc) and ```guis.rs```. It runs a thread that listens for commands from the gui, then sends the RPC requests to the board, then processes the response from the board. It utilizes the Gen3Board struct defined in gen3_rpc/server.rs and the associated traits and methods. 
3. ```main.rs``` is the primary run file in the repo. Anytime a new script with a struct is added to /src, it must be called upon as a module in ```main.rs```. This file is designed to run ```gui.rs```

#### Note: This project does not have full functionality yet 






