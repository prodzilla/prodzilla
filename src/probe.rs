// as:
// call: 
// with: 
//     body:
//     headers:
// expectBack:
//     statusCode: 
//     body:
//     headers:
// schedule:




pub struct Probe {
    pub name: String,
    pub url: String,
    pub with: ProbeInputParameters,
    pub expectBack: ProbeExpectParameters,
    pub schedule: ProbeScheduleParameters,
}

pub struct ProbeInputParameters {
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub struct ProbeExpectParameters {
    pub statusCode: String,
    pub body: String,
}

pub struct ProbeScheduleParameters {
    pub initialDelay: String,
    pub period: String,
}