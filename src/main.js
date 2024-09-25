const { invoke } = window.__TAURI__.tauri;

let algoSelect;
let origTable;
let totalCount = 0;

function makeTable(time_map) {
  let timeTable = document.querySelector("#time-table").content.cloneNode(true);
  let rowInsertPoint = timeTable.querySelector("tbody");
  
  const cellClass = "p-1 border-2 border-solid border-black"

  for (const [process, times] of Object.entries(time_map)) {
    let newRow = document.createElement("tr");
    let process_td = document.createElement("td");
    process_td.className = cellClass
    process_td.textContent = process;

    let waiting_time_td = document.createElement("td");
    waiting_time_td.className = cellClass;
    waiting_time_td.textContent = times.waiting;

    let completion_time_td = document.createElement("td");
    completion_time_td.className = cellClass;
    completion_time_td.textContent = times.completion;

    let turnaround_time_td = document.createElement("td");
    turnaround_time_td.className = cellClass;
    turnaround_time_td.textContent = times.turnaround;

    newRow.append(process_td);
    newRow.append(waiting_time_td);
    newRow.append(turnaround_time_td);
    console.log(process)
    console.log(times)
    newRow.append(completion_time_td);
    rowInsertPoint.append(newRow);
  }
  
  return timeTable
}

function makeChart(segments) {
  let ganttChart = document
    .querySelector("#gantt-chart")
    .content.cloneNode(true);
  let procInsert = ganttChart.querySelector("#gantt-process");
  // procInsert.classList.add(`grid-rows-${res.length - 1}`)
  let timeInsert = ganttChart.querySelector("#time-process");
  // timeInsert.classList.add(`grid-rows-${res.length}`)
  {
    const newTimeBlock = document.createElement("td");
    newTimeBlock.className = "min-w-0 min-h-0 w-10";
    newTimeBlock.innerText = 0;
    timeInsert.append(newTimeBlock);
  }
  segments.forEach((segment) => {
    const newTimeBlock = document.createElement("td");
    newTimeBlock.className = "w-10";
    newTimeBlock.innerText = segment.end_time;
    timeInsert.append(newTimeBlock);

    const newProcBlock = document.createElement("td");
    newProcBlock.className = "w-10 border-2 border-solid border-black";
    if (segment.process_id !== null) {
      newProcBlock.innerText = segment.process_id;
    } else {
      newProcBlock.classList.add("bg-slate-500");
    }
    procInsert.append(newProcBlock);
  });
  return ganttChart;
}

async function handleForm(event) {
  event.preventDefault();
  let text = await invoke("get_state", {});

  // TODO: Finish implementing the DOM manipulation and implement in backend as well

  switch (text) {
    case "SJF": {
      let processes = [];
      for (let i = 1; i <= totalCount; i++) {
        let pName = `p-${i}`;
        let pBt = `bt-${i}`;
        let pAt = `at-${i}`;
        let val = [
          document.getElementById(pName).value,
          parseInt(document.getElementById(pBt).value),
          parseInt(document.getElementById(pAt).value),
        ];
        processes.push(val);
      }

      let [res, time_vals] = await invoke("get_gantt_sjf", { inputs: processes });

      let answerContent = document.querySelector("#answer");
      let ganttChart = makeChart(res);
      answerContent.append(ganttChart);
      let processTable = makeTable(time_vals);
      answerContent.append(processTable);
      break;
    }
    case "FCFS": {
      let processes = [];
      for (let i = 1; i <= totalCount; i++) {
        let pName = `p-${i}`;
        let pBt = `bt-${i}`;
        let pAt = `at-${i}`;
        let val = [
          document.getElementById(pName).value,
          parseInt(document.getElementById(pBt).value),
          parseInt(document.getElementById(pAt).value),
        ];
        processes.push(val);
      }

      let [res, time_vals] = await invoke("get_gantt_fcfs", { inputs: processes });

      let answerContent = document.querySelector("#answer");
      let ganttChart = makeChart(res);
      answerContent.append(ganttChart);
      let processTable = makeTable(time_vals);
      answerContent.append(processTable);
      break;
    }
    case "PRIORITY":{
      let processes = [];
      for (let i = 1; i <= totalCount; i++) {
        let pName = `p-${i}`;
        let pBt = `bt-${i}`;
        let pAt = `at-${i}`;
        let pPrior = `prior-${i}`;
        let val = [
          document.getElementById(pName).value,
          parseInt(document.getElementById(pBt).value),
          parseInt(document.getElementById(pAt).value),
          parseInt(document.getElementById(pPrior).value),
        ];
        processes.push(val);
      }

      let [res, time_vals] = await invoke("get_gantt_priority", { inputs: processes });

      let answerContent = document.querySelector("#answer");
      let ganttChart = makeChart(res);
      answerContent.append(ganttChart);
      let processTable = makeTable(time_vals);
      answerContent.append(processTable);
      break;
    }

    case "RR": {
      let processes = [];
      for (let i = 1; i <= totalCount; i++) {
        let pName = `p-${i}`;
        let pBt = `bt-${i}`;
        let pAt = `at-${i}`;
        let val = [
          document.getElementById(pName).value,
          parseInt(document.getElementById(pBt).value),
          parseInt(document.getElementById(pAt).value),
        ];
        processes.push(val);
      }
      
      let quantum = parseInt(document.getElementById("quantum").value);

      let [res, time_vals] = await invoke("get_gantt_rr", { quantum: quantum, inputs: processes });

      let answerContent = document.querySelector("#answer");
      let ganttChart = makeChart(res);
      answerContent.append(ganttChart);
      let processTable = makeTable(time_vals);
      answerContent.append(processTable);
      break;
    }
    default:
      break;
  }
}

async function addRow() {
  totalCount += 1;
  console.log("Total Count: " + totalCount);
  let tableElement = document.querySelector("table");
  let rowTemplate = document
    .querySelector("#processrow")
    .content.cloneNode(true);

  console.log(rowTemplate);
  let text = await invoke("get_inputs", {});
  let priorityElement = document
    .getElementById("prioritycell")
    .content.cloneNode(true);

  if (text === "Priority") {
    let trInside = rowTemplate.querySelector("tr");
    trInside.append(priorityElement);
  }

  let inputs = rowTemplate.querySelectorAll("input");

  inputs.forEach((element) => {
    element.id = element.id.substring(0, element.id.length - 1) + totalCount;
  });

  let insertPoint = tableElement.querySelector("tbody");
  let addMoreRow = tableElement.querySelector("#addmore");
  insertPoint.insertBefore(rowTemplate, addMoreRow);
}

function generateTable(text) {
  let mainContent = document.querySelector("#content");
  mainContent.innerHTML = "";

  let answerContent = document.querySelector("#answer");
  answerContent.innerHTML = "";

  if (text === "Quantum") {
    let quantumInput = document
      .querySelector("template#quantuminput")
      .content.cloneNode(true);
    mainContent.append(quantumInput);
  }

  let tableTemplate = document
    .getElementById("process-table")
    .content.cloneNode(true);

  if (text === "Priority") {
    let priorityElement = document
      .getElementById("priorityinput")
      .content.cloneNode(true);
    let insertPoint = tableTemplate.querySelector("tr");
    insertPoint.append(priorityElement);
  }

  let rowTemplate = document
    .getElementById("processrow")
    .content.cloneNode(true);

  if (text === "Priority") {
    let priorityElement = document
      .getElementById("prioritycell")
      .content.cloneNode(true);
    let insertPoint = rowTemplate.querySelector("tr");
    insertPoint.append(priorityElement);
  }

  let tableElement = tableTemplate.querySelector("tbody");
  let addMoreRow = tableElement.querySelector("#addmore");
  tableElement.insertBefore(rowTemplate, addMoreRow);

  mainContent.append(tableTemplate);
  document.querySelector("tr#addmore button").addEventListener("click", addRow);
  document
    .querySelector("#process-form")
    .addEventListener("submit", handleForm);
  totalCount = 1;
}

window.addEventListener("DOMContentLoaded", () => {
  algoSelect = document.querySelector("#algo");
  algoSelect.addEventListener("change", (e) => {
    e.preventDefault();
    invoke("change_state", { algo: algoSelect.value }).then((res) => {
      generateTable(res);
    });
  });
});
