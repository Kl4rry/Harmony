function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function drop(ev) {
    ev.preventDefault();
    external.invoke(ev.target.innerHTML);
    external.invoke("data: " + ev.dataTransfer.getData("item"));
}

function drag(ev) {
    ev.dataTransfer.setData("item", ev.target);
    external.invoke("drag");
}

async function dragover(ev) {
    let target = ev.target;
    while(target.className != "item") {
        let target = target.parentNode;
    }
    ev.preventDefault();
    ev.target.id = "target";
}

function leave(ev) {
    external.invoke("leave");
    if(ev.target.id == "target") {
        ev.target.id = "";
    }
}

function enter(ev) {
    external.invoke("enter");
}

function browse() {
    external.invoke("browse");
}

function play_pause(button) {
    external.invoke(`play_pause ${button.parentNode.parentNode.id}`);
}

function restart(button){
    external.invoke(`restart ${button.parentNode.parentNode.id}`);
}

function new_sound(id, name) {
    let div = document.createElement("div");
    div.setAttribute("class", "item");
    div.setAttribute("id", id);
    div.innerHTML = `<p>Loading: ${name}</p>`;
    document.getElementById("main").appendChild(div);
}

function init_sound(id, name, duration) {
    let div = document.getElementById(id);
    let play = `<div class="button-container"><div class="item-button" onclick="play_pause(this)"><div id="play" class="play-icon"></div></div></div>`
    let restart = `<div class="button-container"><div class="item-button" onclick="restart(this)"><div class="play-icon"></div></div></div>`
    div.setAttribute("ondragstart", "drag(event)");
    div.setAttribute("ondragover", "dragover(event)");
    div.setAttribute("ondrop", "drop(event)");
    div.setAttribute("ondragleave", "leave(event)");
    div.setAttribute("dragenter", "enter(event)");
    div.setAttribute("draggable", "true");
    div.innerHTML = `${play}${restart}<p class="name">${name}</p><p class="duration">${duration}</p>`;
    
}

function set_icon(id, icon) {
    let div = document.getElementById(id);
    let img = div.querySelector("#play");
    img.className = icon;
}

function on_end(id) {
    set_icon(id, "play-icon");
}

function remove_sound(id) {
    let div = document.getElementById(id);
    document.getElementById("main").removeChild(div);
}

function set_device_list(list) {
    let inner = "";
    for(let i = 0; i<list.length; ++i){
        inner += `<option value=${i}>${list[i]}</option>`;
    }
    document.getElementById("primary-devicelist").innerHTML = inner;
    document.getElementById("secondary-devicelist").innerHTML = inner;
}

function select_primary(selectedIndex) {
    external.invoke(`select_primary ${selectedIndex}`);
}

function select_secondary(selectedIndex) {
    external.invoke(`select_secondary ${selectedIndex}`);
}

function update_device_list() {
    external.invoke("update_device_list");
}

let primary_volume = 0.5;
let secondary_volume = 0.5;

function set_primary_volume(volume) {
    secondary_volume = volume;
    external.invoke(`set_volume ${primary_volume} ${secondary_volume}`);
}

function set_secondary_volume(volume) {
    primary_volume = volume;
    external.invoke(`set_volume ${primary_volume} ${secondary_volume}`);
}

update_device_list();