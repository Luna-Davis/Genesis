# Genesis - Project Manager

Genesis is a project manager built for my specific workflow with Python, Rust, Flutter and JavaScript. I contains three terminal commands
`new`, `init`, and `shell`. These work together for the efficient management of a project.

## Features

Genesis has a shell that only works inside a Genesis project identified by `.genesis.json` inside the project root directory. This 
shell has a number of commands built in that are displayed using the `help` command. 

The commands include:

<table>
<tr>
<th>Command</th>
<th>Function</th>
</tr>

<tr>
<td>file</td>
<td>File manipulation; Creation and Deletion</td>
<tr>

<tr>
<td>module</td>
<td>Module manipulation; Creation and Deletion</td>
</tr>

<tr>
<td>run</td>
<td>Runs the project</td>
</tr>

<tr>
<td>test</td>
<td>Runs the tests for the project</td>
</tr>

<tr>
<td>dev</td>
<td>Runs a dev server (Flutter specific)</td>
</tr>

<tr>
<td>build</td>
<td>Builds the project for a specific platform (Flutter specific)</td>
</tr>

<tr>
<td>help</td>
<td>Displays help - obviously</td>
</tr>

<tr>
<td>exit/quit</td>
<td>Quits genesis</td>
</tr>

<tr>
<td>clear</td>
<td>Clears the screen of the project</td>
</tr>
</table>

## Installation

To install genesis just run the following commands:

```bash
git clone https://codeberg.org/luna-davis/Genesis.git
cd Genesis
./build.sh 
```
This builds the project and places the binary inside `~/.local/bin`

| **NOTE:** The project isn't platform agnostic yet


## Contribution

To contribute, just clone the repository and make modifications then submit a pull request and 
I'll review the code written we can work on this together
