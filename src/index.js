function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}  

function browse(){
    external.invoke("browse");
}

function play(button){
    external.invoke(`play ${button.parentNode.id}`);
}

function new_sound(id, name){
    let div = document.createElement("div");
    div.setAttribute("class", "item");
    div.setAttribute("id", id);
    div.innerHTML = `<p>Loading: ${name}</p>`;
    document.getElementById("main").appendChild(div);
    
}

function init_sound(id, name){
    let div = document.getElementById(id);
    div.innerHTML = `<button onclick="play(this)">Play</button><p>${name}</p>`;
}