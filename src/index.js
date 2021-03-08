function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

let dragged = null;

function drop(ev) {
    ev.preventDefault();
    dragged = null;
}

function drag(ev) {
    let rect = ev.target.getBoundingClientRect();
    let x = ev.clientX - rect.left;
    let y = ev.clientY - rect.top;
    let width = ev.target.offsetWidth;

    dragged = ev.target;
    const crt = dragged.cloneNode(true);
    crt.style.opacity = "0.2";
    crt.style.minWidth = `${width}px`;
    crt.style.position = "absolute";
    crt.style.top = "0px";
    crt.style.zIndex = "-10";
    crt.id = "ghost";
    crt.style.right = "0px";
    document.body.appendChild(crt);
    ev.dataTransfer.setDragImage(crt, x, y);
    document.body.style.cursor = "grabbing";
}

async function dragover(ev) {
    ev.dataTransfer.dropEffect = "move";
    let target = ev.target;
    let target_index = Array.from(target.parentNode.children).indexOf(target);
    let dragged_index = Array.from(dragged.parentNode.children).indexOf(dragged);
    if(target.classList.contains("item")) {
        ev.preventDefault();
        if(target_index > dragged_index) {
            target.parentNode.insertBefore(dragged, target.nextSibling);
        } else if(target_index < dragged_index) {
            target.parentNode.insertBefore(dragged, target);
        }
    }
}

function dragend(ev) {
    let ghost = document.getElementById("ghost");
    if(ghost && ghost.parentNode) {
        ghost.parentNode.removeChild(ghost);
    }
    document.body.style.cursor = null;
}

function browse() {
    close_all_menus();
    external.invoke("browse");
}

function exit() {
    external.invoke("exit");
}

//menu bar
let menu_open = false;

function show_menu(name) {
    let menu = document.getElementById(`${name}-menu`);
    if(menu_open) {
        menu.style.display = "none";
        document.getElementById(`${name}-button`).style.backgroundColor = "#21252B";
        document.getElementById("menu-bar-close-zone").style.display = "none";
        menu_open = false;
    } else {
        menu.style.display = "block";
        document.getElementById(`${name}-button`).style.backgroundColor = "#383B41";
        document.getElementById("menu-bar-close-zone").style.display = "block";
        menu_open = true;
    }
}

function mouse_over_menu(name) {
    if(menu_open) {
        close_all_menus();
        show_menu(name);
    }
}

function close_all_menus() {
    document.getElementById("file-menu").style.display = "none";
    document.getElementById("file-button").style.backgroundColor = "#21252B";

    document.getElementById("edit-menu").style.display = "none";
    document.getElementById("edit-button").style.backgroundColor = "#21252B";

    document.getElementById("menu-bar-close-zone").style.display = "none";
    menu_open = false;
}

//sounditems
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

    div.setAttribute("ondragstart", "drag(event)");
    div.setAttribute("ondragover", "dragover(event)");
    div.setAttribute("ondrop", "drop(event)");
    div.setAttribute("ondragend", "dragend(event)");
    div.setAttribute("draggable", "true");
    div.setAttribute("onclick", "select(event)");

    div.innerHTML = `<p>Loading: ${name}</p>`;
    document.getElementById("grid").appendChild(div);
}

function init_sound(id, name, duration) {
    let div = document.getElementById(id);
    let play = `<div class="button-container"><div class="item-button" onclick="play_pause(this)"><div id="play" class="play-icon"></div></div></div>`
    let restart = `<div class="button-container"><div class="item-button" onclick="restart(this)"><div class="play-icon"></div></div></div>`
    div.innerHTML = `${play}${restart}<p class="name">${name}</p><p class="duration">${duration}</p>`;
    div.setAttribute("init", true);
}

function set_icon(id, icon) {
    let div = document.getElementById(id);
    let img = div.querySelector("#play");
    img.className = icon;
}

function on_end(id) {
    set_icon(id, "play-icon");
}

//just removes the ui element
function remove_sound_item(id) {
    let div = document.getElementById(id);
    document.getElementById("grid").removeChild(div);
}


//removes ui and backend audio clip if it is inited
async function remove_sound(id) {
    let div = document.getElementById(id);
    if(div.getAttribute("init")) {
        document.getElementById("grid").removeChild(div);
        external.invoke(`remove ${id}`)
    }
}

async function remove_selected() {
    close_all_menus();
    let selected = document.getElementsByClassName("selected");
    let ids = [];
    for(let i = 0; i < selected.length; ++i) {
        ids.push(selected[i].id)
    }
    for(let i = 0; i < ids.length; ++i) {
        remove_sound(ids[i]);
    }
}

//selection
async function select(event) {
    let items = document.getElementsByClassName("item");
    let target = event.target;
    while(!target.classList.contains("item")) {
        target = target.parentNode;
    }
    if(event.shiftKey) {
        let selected = document.getElementsByClassName("selected");
        let first_index = Array.from(items).indexOf(selected[0]);
        let click_index = Array.from(items).indexOf(target);

        for(let i = 0; i < items.length; ++i) {
            items[i].classList.remove("selected");
        }

        if(first_index > click_index) {
            for(let i = click_index; i < first_index; ++i) {
                items[i].classList.add("selected");
            }
        } else {
            for(let i = first_index; i < click_index; ++i) {
                items[i].classList.add("selected");
            }
        }
    } else if(!event.ctrlKey) {
        for(i = 0; i < items.length; i++) {
            items[i].classList.remove("selected");
        }
    }
    target.classList.add("selected");
}

//devices and volume settings
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

function update_volume(primary, secondary) {
    document.getElementById("primary-slider").value = primary;
    primary_volume = primary;
    document.getElementById("secondary-slider").value = secondary;
    secondary_volume = secondary;
}

function update_device(primary, secondary) {
    document.getElementById("primary-devicelist").value = primary;
    document.getElementById("secondary-devicelist").value = secondary;
}

update_device_list();
external.invoke("ready");