let questionId = 1;
let points = 0;
let addpoints = 5;
let maxpoints = 50;
let oldpoints = 0;
let helped1 = 0;
let helped2 = 0;
let points_per_question = new Array(10).fill(0);

function getTextWidth(text, font) {
  let canvas = getTextWidth.canvas || (getTextWidth.canvas = document.createElement("canvas"));
  let context = canvas.getContext("2d");
  context.font = font;
  let metrics = context.measureText(text);
  return metrics.width;
}

function save_cache() {
    sessionStorage.setItem("questionId", questionId);
    sessionStorage.setItem("saved", true);
    sessionStorage.setItem("nextBookButtonState", document.getElementById("nextBookButton").style.visibility);
    //sessionStorage.setItem("nextBookButtonState", document.getElementById("nextBookButton").hidden)
}

function load_cache() {
    if (!sessionStorage.saved) return;
    questionId = Number(sessionStorage.getItem("questionId"));
    document.getElementById("nextBookButton").style.visibility = String(sessionStorage.getItem("nextBookButtonState"));
    //document.getElementById("nextBookButton").hidden = Boolean(sessionStorage.getItem("nextBookButtonState"))
}

function get_id() {
    const queryString = window.location.search;
    const urlParams = new URLSearchParams(queryString);

    var id = urlParams.get("id");
    if (id == null) {
        return 0;
    } else {
        return id;
    }
}

function blurTemplate(sentence) {
    const template =
        `<blur
        onclick="blurOnClick(this)"
    >`
    return `${template} ${sentence}</blur>`;
}

function unblur(element) {
    element.style.filter = "blur(0px)";
    addpoints--;
}

function blurOnClick(element) {
    element.style.color = "black";
    unblur(element);
    element.class = "blur_disabled";
}

function httpGet(url) {
    var xmlHttp = new XMLHttpRequest();
    xmlHttp.open("GET", url, false); // false for synchronous request
    xmlHttp.send(null);
    return xmlHttp.responseText;
}

function set_sentences() {
    let sentences = [
        httpGet(`/sentence?id=${questionId}&s=Sentence${1}`).replace(/['"]+/g, ''),
        httpGet(`/sentence?id=${questionId}&s=Sentence${2}`).replace(/['"]+/g, ''),
        httpGet(`/sentence?id=${questionId}&s=Sentence${3}`).replace(/['"]+/g, '')
    ];
    document.getElementById("sen").innerHTML = `
            ${sentences[0]}
            ${blurTemplate(sentences[1])}
            ${blurTemplate(sentences[2])}`;
}

function get_book() {
    var url = `/title?id=${questionId}`;
    var res = httpGet(url);
    var json = JSON.parse(res);

    return json;
}

function get_book_by_id(id) {
    var url = `/title?id=${id}`;
    var res = httpGet(url);
    var json = JSON.parse(res);

    return json;
}

function check_book() {
    var book = get_book();
    var guess = document.getElementById('frm');
    var guessButton = document.getElementById("button1");
    var giveUpButton = document.getElementById("button2");
    var input = document.getElementById("field");
    var help1 = document.getElementById("help1_button");
    var help2 = document.getElementById("help2_button");

    if (guess.elements[0].value.toUpperCase().trim() === book.title.replace(/['"]+/g, '').toUpperCase() ||
        guess.elements[0].value.toUpperCase().trim() === book.title_en.replace(/['"]+/g, '').toUpperCase()) {
        giveUpButton.hidden = true;
        guessButton.disabled = true;
        input.readOnly = true;
        help1.style.visibility = 'hidden';
        help2.style.visibility = 'hidden';

        document.getElementById("sen").innerHTML = "<p style='text-align: center'> Dobrze! </p>" +
            "<h3 style='text-align: center'>" + book.title.replace(/['"]+/g, '') + "</h3><p style='text-align: center'>" + book.author.replace(/['"]+/g, '') + "</p>";
        addpoints = (addpoints > 0) ? addpoints : 1;
        oldpoints = points;
        points += addpoints;
        points_per_question[questionId-1] = addpoints;
        addpoints = 5;
        show_points();
        document.getElementById("nextBookButton").style.visibility = 'visible';
    } else {
        guess.classList.add("apply-shake");
        //document.getElementById("title").innerHTML = "<h3>"+guess.elements[0].value + " to błędna odpowiedź. Spróbuj ponownie!</h3> <br>";
        guess.addEventListener("animationend", (e) => {
            guess.classList.remove("apply-shake");
        });
        addpoints -= 2;
    }
}

function show_points() {
    let progressBarAnimation = [{
            strokeDashoffset: (600) - ((600) * (36 * oldpoints / maxpoints)) / 100
        },
        {
            strokeDashoffset: (600) - ((600) * (36 * points / maxpoints)) / 100
        }
    ];
    document.getElementById('complete-bar').animate(progressBarAnimation, {
        duration: 250
    });
    document.getElementById('complete-bar').style.strokeDashoffset = (600) - ((600) * (36 * points / maxpoints)) / 100;
    document.getElementById("points").textContent = points;
}

function give_up() {
    var book = get_book();

    var guessButton = document.getElementById("button1");
    var help1 = document.getElementById("help1_button");
    var help2 = document.getElementById("help2_button");

    help1.style.visibility = 'hidden';
    help2.style.visibility = 'hidden';
    guessButton.disabled = true;
    guessButton.style.visibility = 'hidden';
    document.getElementById("nextBookButton").style.visibility = 'visible';
    var giveUpButton = document.getElementById("button2");
    giveUpButton.hidden = true;
    //giveUpButton.parentNode.removeChild(giveUpButton);


    var input = document.getElementById("field");
    input.readOnly = true;

    document.getElementById("sen").innerHTML = "<p style='text-align: center'> Twoja książka to: </p>" + "<h3 style='text-align: center'>" + book.title.replace(/['"]+/g, '') + "</h3><p style='text-align: center'>" + book.author.replace(/['"]+/g, '') + "</p>";

}

function get_max_id() {
    var url = "/books-counter";
    var maxId = parseInt(httpGet(url));

    return maxId;
}

function reset_form() {
    document.getElementById("frm").reset();
    document.getElementById("button2").hidden = false;
    document.getElementById("button1").disabled = false;
    document.getElementById("button1").style.visibility = 'visible';
    document.getElementById("field").readOnly = false;
    document.getElementById("nextBookButton").style.visibility = 'hidden';
    if (helped1 === 0) {
        document.getElementById('help1_button').style.visibility = 'visible';
    } else {
        document.getElementById('help1_button').style.visibility = 'hidden';
        document.getElementById('help1_button').disabled = true;
    }
    if (helped2 === 0) {
        document.getElementById('help2_button').style.visibility = 'visible';
    } else {
        document.getElementById('help2_button').style.visibility = 'hidden';
        document.getElementById('help2_button').disabled = true;
    }
    var id = questionId;
    var max_id = get_max_id();
    if (id === max_id) {
        document.getElementById("next").hidden = "true";
        document.getElementById('nextBookButton').style.visibility = 'hidden';
    }
}

function get_help1() {
    var help = get_book().ganre.replace(/['"]+/g, '');
    var textWidth = getTextWidth(help, "bold 15px Arial");
    var margin = (155.11 - textWidth) * (-1);
    var selfMargin = (-textWidth + 83.669 - textWidth/4);
    if (margin < -85.11 ){
        margin = -85.11;
        selfMargin = 0;
    }
    console.log(margin);
    document.getElementById("help1_button").style.marginLeft = margin;//"-20px";
    document.getElementById("help1").innerHTML = `<span style='margin-left: ${selfMargin}'><strong>` + help + "</strong></span>";
    addpoints--;
    helped1 = 1;
}

function get_help2() {
    var help = get_book().author.replace(/['"]+/g, '');
    var textWidth = getTextWidth(help, "bold 15px Arial");
    var margin = (155.11 - textWidth) * (-1);
    var selfMargin = (-textWidth + 83.669 - textWidth/3);
    if (margin < -85.11) {
        margin = -85.11;
        selfMargin = -textWidth/3;
    }

    console.log(margin);
    document.getElementById("help2_button").style.marginLeft = margin //"-20px";
    document.getElementById("help2").innerHTML = `<span style='margin-left: ${selfMargin}'><strong>` + help + "</strong></span>";
    addpoints--;
    helped2 = 1;
}

function next_book_test() {
    let maxId = get_max_id();
    let nextId = questionId + 1;
    //if (nextId >= maxId) {
        ////document.getElementById("nextBookButton").hidden = true;
        //document.getElementById("nextBookButton").onclick = show_result_screen();
    //}
    if (questionId === maxId){
        show_result_screen();
    }
    else {
        reset_form();
        questionId++;
        set_sentences();
        save_cache();
    }
}

function next_book() {
    var id = parseInt(get_id());
    var maxId = get_max_id();
    var newId = id + 1;
    if (newId <= maxId) {
        location.href = `/?id=${newId}`;
    }

}

function show_result_screen()
{
    var screen = document.getElementById("sen");
    screen.innerHTML = "<p style='text-align: center'> Końcowy wynik: </p>";
    //screen.innerText = "";

    for (let i = 1; i <= 10; i++)
    {
        let book = get_book_by_id(i);
        //screen.innerText += book.title.replaceAll('"', '');
        screen.style.fontSize = "26px";
        //screen.style.lineHeight = "50px";

        if (points_per_question[i-1] > 0) {
            screen.innerHTML += "<p style='color:green'> ✓ " + book.author.replaceAll('"', '') + " " + book.title + " " + points_per_question[i-1] + "/5 pkt</p>";
        }
        else {
            screen.innerHTML += "<p style='color:red'> ✗ " + book.author.replaceAll('"', '') + " " + book.title + " " + points_per_question[i-1] + "/5 pkt</p>";
        }
        //screen.innerHTML += " " + book.author.replaceAll('"', '') + " " + book.title;
        //screen.innerHTML += " " + points_per_question[i-1]+"/5 </p>";

        //screen.innerText += "\n";
    }
    document.getElementById("nextBookButton").hidden = true;
    document.getElementById("button1").hidden = true;
    document.getElementById("field").hidden = true;
}

const animationTiming = {
    duration: 350,
    iterations: 1,
}
