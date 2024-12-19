const textarea = document.getElementById("textarea")

document.getElementById("login").addEventListener("click", async (evt) => {
    let loginResponse = await fetch("/api/login", {
        method: "POST",
        body: JSON.stringify(gatherLogin()),
    })

    if (loginResponse.ok) {
        loginResponse = await loginResponse.json()
        window.localStorage.setItem("authority", loginResponse.authority)
    }

    textarea.value = JSON.stringify(loginResponse, null, 4)
})

document.getElementById("create").addEventListener("click", async (evt) => {
    const loginResponse = await fetch("/api/user", {
        method: "POST",
        body: JSON.stringify(gatherLogin()),
    }).then((r) => r.json())

    textarea.value = JSON.stringify(loginResponse, null, 4)
})

document.getElementById("gettask")

document.getElementById("newtask").addEventListener("click", async () => {
    const authority = window.localStorage.getItem("authority")
    const headers = new Headers()
    headers.append("authority", authority)
    const userRespone = await fetch("/api/user", {
        headers,
    }).then((r) => r.json())

    console.log({ userRespone })

    const body = {
        due_date: document.getElementById("inputdate").value,
        assign_date: new Date(Date.now()).toISOString().slice(0, 10),
        title: document.getElementById("inputtitle").value,
        description: document.getElementById("inputdesc").value,
        user_id: userRespone.userId,
    }

    console.log(body)

    const r = await fetch("/api/task", {
        method: "POST",
        headers: headers,
        body: JSON.stringify(body),
    }).then((r) => r.json())

    textarea.value = JSON.stringify(r, null, 4)
})

document.getElementById("deauth").addEventListener("click", () => {
    window.localStorage.setItem("authority", "")
})

function gatherLogin() {
    const username = document.getElementById("userinput").value
    const password = document.getElementById("passwordinput").value
    return {
        username,
        password,
    }
}