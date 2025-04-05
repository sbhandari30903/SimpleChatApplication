var ws;
var userId = null;
var selectedUserId = null;
var users = [];

function connectWebSocket(currentUserId) {
	if (!currentUserId) {
		console.log("User ID not available yet, delaying WebSocket connection.");
		return;
	}
	ws = new WebSocket("ws://localhost:8082/ws?userId=" + currentUserId);

	ws.onopen = function () {
		console.log("WebSocket connection established");
	};

	ws.onmessage = function (event) {
		var message = JSON.parse(event.data);
		console.log(" message received is ", message);

		if (message.sender_id === selectedUserId || message.receiver_id === userId) {
			displayMessage(message);
		}
	};

	ws.onerror = function (error) {
		console.error("WebSocket error:", error);
	};

	ws.onclose = function () {
		console.log("WebSocket connection closed, attempting to reconnect...");
	};
}

function registerUser() {
	var firstName = document.getElementById("registerFirstName").value;
	var lastName = document.getElementById("registerLastName").value;

	fetch("http://localhost:8082/register", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			first_name: firstName,
			last_name: lastName,
		}),
	})
		.then((response) => response.json())
		.then((user) => {
			alert("User registered successfully!");
		})
		.catch((error) => console.error("Error registering user:", error));
}

function loginUser() {
	var firstName = document.getElementById("loginFirstName").value;
	var lastName = document.getElementById("loginLastName").value;

	fetch("http://localhost:8082/login", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			first_name: firstName,
			last_name: lastName,
		}),
	})
		.then((response) => response.json())
		.then((user) => {
			console.log("Logged in as " + user.first_name + " " + user.last_name);
			userId = user.id;
			alert("Logged in as " + user.first_name + " " + user.last_name);

			document.getElementById("loggedInUser").innerHTML = `
			<img id="userAvatar" src="https://substackcdn.com/image/fetch/f_auto,q_auto:good,fl_progressive:steep/https%3A%2F%2Fbucketeer-e05bbc84-baa3-437e-9518-adb32be77984.s3.amazonaws.com%2Fpublic%2Fimages%2Fa44f58da-a39d-4e71-807b-9332c39a2be1_976x549.jpeg" alt="User Avatar" />
			<span id="userName">Hi, ${user.first_name} ${user.last_name}</span>
		`;
			document.getElementById("registerSection").style.display = "none";
			document.getElementById("loginSection").style.display = "none";
			document.getElementById("chatSection").style.display = "block";

			connectWebSocket(userId);
			fetchUsers();
		})
		.catch((error) => alert("Login failed: " + error.message));
}

function fetchUsers() {
	fetch("http://localhost:8082/users")
		.then((response) => response.json())
		.then((data) => {
			users = data;
			updateUserList();
		})
		.catch((error) => console.error("Error fetching users:", error));
}

function updateUserList() {
    var userListItems = document.getElementById("userListItems");
    userListItems.innerHTML = "";
    users.forEach((user) => {
        if (user.id === userId) return;

        var listItem = document.createElement("li");
        listItem.style.cursor = "pointer";
        listItem.onclick = function () {
            selectedUserId = user.id;
            highlightSelectedUser(listItem);
            fetchMessagesForUser(selectedUserId);

			document.getElementById("selectedUserIdHeader").innerText = user.first_name
        };

        // Create an image element for the avatar
        var avatar = document.createElement("img");
        avatar.src = 'https://gratisography.com/wp-content/uploads/2024/11/gratisography-augmented-reality-800x525.jpg';  // assuming `avatarUrl` contains the link to the user's avatar image
        avatar.alt = `${user.first_name} ${user.last_name}'s Avatar`;
        avatar.style.width = "40px";  // adjust size as needed
        avatar.style.height = "40px"; // adjust size as needed
        avatar.style.marginRight = "10px";  // space between the avatar and the name

        // Create a span for the user's name
        var userName = document.createElement("span");
        userName.textContent = `${user.first_name} ${user.last_name}`;

        // Append the avatar and name to the list item
        listItem.appendChild(avatar);
        listItem.appendChild(userName);

        // Append the list item to the user list
        userListItems.appendChild(listItem);
    });
}


function highlightSelectedUser(selectedElement) {
	const listItems = document.querySelectorAll("#userListItems li");
	listItems.forEach((item) => {
		item.style.fontWeight = "normal";
		item.style.backgroundColor = "transparent";
	});
	selectedElement.style.fontWeight = "bold";
	selectedElement.style.backgroundColor = "#d3e0f5";
}

function fetchMessagesForUser(selectedUserId) {
	fetch(
		`http://localhost:8082/messages?userId1=${userId}&userId2=${selectedUserId}`
	)
		.then((response) => response.json())
		.then((messages) => {
			displayMessages(messages);
		})
		.catch((error) => console.error("Error fetching messages:", error));
}

function displayMessage(message) {
	var messageContainer = document.getElementById("messages");
	var messageElement = document.createElement("div");
	messageElement.classList.add("message");
	messageElement.textContent = `${message.content}`;
	messageContainer.appendChild(messageElement);
	messageContainer.scrollTop = messageContainer.scrollHeight;
}

function displayMessages(messages) {
	var messageContainer = document.getElementById("messages");
	messageContainer.innerHTML = "";
	messages.sort((a, b) => new Date(a.timestamp) - new Date(b.timestamp));
	messages.forEach((message) => {
		var messageElement = document.createElement("div");
		messageElement.classList.add("message");
		if(selectedUserId == message.sender_id){
			messageElement.classList.add("message-right");
		}
		messageElement.textContent = `${message.content}`;
		messageContainer.appendChild(messageElement);
	});
	messageContainer.scrollTop = messageContainer.scrollHeight;
}

function sendMessage(e) {
	e.preventDefault();

	var messageContent = document.getElementById("messageInput").value;

	if (!selectedUserId || !messageContent) {
		alert("Please select a user and type a message.");
		return;
	}

	var messageData = {
		sender_id: userId,
		receiver_id: selectedUserId,
		content: messageContent,
	};

	var messageElement = document.createElement("div");
	messageElement.classList.add("message");
	messageElement.classList.add("message-right");
	messageElement.textContent = `You: ${messageContent}`;
	document.getElementById("messages").appendChild(messageElement);

	document.getElementById("messageInput").value = "";
	if (ws && ws.readyState === WebSocket.OPEN) {
		ws.send(JSON.stringify(messageData));
		document.getElementById("messageInput").value = "";
	} else {
		alert("WebSocket is not connected.");
	}
}

document.getElementById("chatForm").addEventListener("submit", sendMessage);
