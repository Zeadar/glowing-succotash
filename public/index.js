const textarea = document.getElementById("textarea")

document.getElementById("login").addEventListener("click", async () => {
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

document.getElementById("create").addEventListener("click", async () => {
    const loginResponse = await fetch("/api/user", {
        method: "POST",
        body: JSON.stringify(gatherLogin()),
    }).then((r) => r.json())

    textarea.value = JSON.stringify(loginResponse, null, 4)
})

document.getElementById("task").addEventListener("click", async () => {
    const authority = window.localStorage.getItem("authority")
    const headers = new Headers()
    headers.append("authority", authority)

    const r = await fetch("/api/task", {
        headers: headers,
    }).then((r) => r.json())

    console.log(r)

    textarea.value = JSON.stringify(r, null, 4)
})

document.getElementById("newtask").addEventListener("click", async () => {
    const authority = window.localStorage.getItem("authority")
    const headers = new Headers()
    headers.append("authority", authority)
    const userResponse = await fetch("/api/user", {
        headers,
    }).then((r) => r.json())

    console.log({ userRespone: userResponse })

    const body = {
        due_date: document.getElementById("inputdate").value,
        assign_date: new Date(Date.now()).toISOString().slice(0, 10),
        title: document.getElementById("inputtitle").value,
        description: document.getElementById("inputdesc").value,
        user_id: userResponse.userId,
        recurring_month: true,
        recurring_n: false,
        recurring_stop: "",
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
