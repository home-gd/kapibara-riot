# Kapibara Riot

[Українська](README.uk.md)

A small platform game. These steps are for Windows. Use GitHub Desktop or git in a terminal. The GitHub Desktop menus are in English.

## Install the tools once

### Rust

Open https://rustup.rs, download `rustup-init.exe`, and run it. When it asks what to do, choose the default install: press `1`, then Enter.

If it says Visual Studio or the C++ build tools are missing, install them before continuing: https://visualstudio.microsoft.com/visual-cpp-build-tools/

In that installer, select the workload **Desktop development with C++**. Then run `rustup-init.exe` again.

### make

1. Press the Windows key.
2. Type `Terminal`. On a Ukrainian Windows, type `Термінал`.
3. Press Enter.
4. In that window, type this and press Enter:

```
winget install ezwinports.make
```

5. Close the terminal when it finishes. Windows does not see new commands in a window that was already open.

### Git, if you are not using GitHub Desktop

Download the installer from https://git-scm.com/download/win and run it. The default options are fine. Leave “Git from the command line and also from 3rd-party software” selected. GitHub Desktop already includes git, so skip this if that is what you use.

## Download the game once

### With GitHub Desktop

1. Open GitHub Desktop.
2. In the menu bar, click **File**, then **Clone repository**.
3. Click the **URL** tab.
4. Paste this address:

```
https://github.com/home-gd/kapibara-riot
```

5. Leave the local path as it is, unless you want another folder. The usual place is `C:\Users\YourName\Documents\GitHub\kapibara-riot`.
6. Click **Clone**.

### With git in the terminal

Open a new terminal, then type these commands one at a time. Press Enter after each one and wait until it finishes.

```
git clone https://github.com/home-gd/kapibara-riot.git
cd kapibara-riot
```

The `kapibara-riot` folder appears in the folder where the terminal started. That is usually `C:\Users\YourName\kapibara-riot`.

## Play

The first start builds the game. That can take several minutes, and the game window opens only after the build finishes. Later starts are faster.

Leave the terminal open while you play. Closing it closes the game.

If the window says `make` is not recognized, the install is not visible yet. Close the window and open a new one.

### With GitHub Desktop

1. Check that the current repository name at the top is **kapibara-riot**.
2. In the menu bar, click **Repository**.
3. Click **Open in PowerShell** or **Open in Command Prompt**. The name depends on a setting in GitHub Desktop. Either one is fine.
4. A window opens in the game folder. Type this and press Enter:

```
make run
```

### With git in the terminal

Open a new terminal. If you just cloned the game and you are still in `kapibara-riot`, type:

```
make run
```

If this is a later visit, go to the folder first:

```
cd kapibara-riot
make run
```

If `cd kapibara-riot` says the folder does not exist, this terminal is not in the folder where you cloned the game. A new terminal starts in your user folder.

## When the game has been updated

Close the game and the window where it was started before you download the update.

### With GitHub Desktop

1. Open GitHub Desktop on **kapibara-riot**.
2. Click **Fetch origin**. If the button changes to **Pull origin**, click that too.
3. Open the game folder again: **Repository**, then **Open in PowerShell** or **Open in Command Prompt**.
4. Type `make run` and press Enter.

### With git in the terminal

```
cd kapibara-riot
git pull
make run
```

## Controls

- Move with A and D, or the left and right arrow keys. Holding both sides at once stands still.
- Jump with Space, W, or the up arrow. A quick tap is a short hop. Hold the key to jump higher.
- To drop through a platform, hold S or the down arrow and press Space. This does not drop through the ground.
