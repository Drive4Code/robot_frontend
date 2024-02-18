<h1>Instructions</h1>
<p>(Preliminary step) Ensure the project directory is in the windows defender exclusion</p>
<ol>
  <li>rustup target add wasm32-unkown-unkown</li>
  <li>cargo install --locked trunk</li>
  <li>trunk serve --release</li>
</ol>

<h1>Note</h1>
<p>A custom version of the worldgen, with the file included named <a href="src/worldloader.rs">worldloader.rs</a>, was included due to the wasm32 target limitations. It has been offcially prvided by the unwrap().unwrap().unwrap() team and uses the include_bytes! macro to embed the map. To select a new map, place it in the worlds folder inside src and change the path inside the macro</p>
