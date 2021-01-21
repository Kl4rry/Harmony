function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}  

function browse(){
    external.invoke("browse");
}

function play(button){
    external.invoke(`play ${button.parentNode.parentNode.id}`);
}

function restart(button){
    external.invoke(`restart ${button.parentNode.parentNode.id}`);
}

function new_sound(id, name){
    let div = document.createElement("div");
    div.setAttribute("class", "item");
    div.setAttribute("id", id);
    div.innerHTML = `<p>Loading: ${name}</p>`;
    document.getElementById("main").appendChild(div);
}

function init_sound(id, name, duration){
    let div = document.getElementById(id);
    let play = `<div class="button-container"><div class="item-button" onclick="play(this)"><div class="play-icon"></div></div></div>`
    let restart = `<div class="button-container"><div class="item-button" onclick="restart(this)"><div class="play-icon"></div></div></div>`
    div.innerHTML = `${play}${restart}<p class="name">${name}</p><p class="duration">${duration}</p>`;
}

function remove_sound(id){
    let div = document.getElementById(id);
    document.getElementById("main").removeChild(div);
}

function set_device_list(list){
    let inner = "";
    for(let i = 0; i<list.length; ++i){
        inner += `<option value=${i}>${list[i]}</option>`;
        external.invoke(list[i]);
    }
    document.getElementById("primary-devicelist").innerHTML = inner;
    document.getElementById("secondary-devicelist").innerHTML = inner;
}

function select_primary(selectedIndex){
    external.invoke(`select_primary ${selectedIndex}`);
}

function select_secondary(selectedIndex){
    external.invoke(`select_secondary ${selectedIndex}`);
}

function update_device_list(){
    external.invoke("update_device_list");
}

update_device_list();