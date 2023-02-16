export interface Request {
    accessToken: string;
    path: string;
    method: HttpMethod;
    body?: Record<string, any>;
}

export enum HttpMethod {
    get = "GET",
    post = "POST",
    put = "PUT",
    delete = "DELETE",
}

export type Organization = {
    id: string;
    userEmail: string;
    name: string;
}

export enum VersionType {
    Backend = "Backend",
    Ide = "Ide",
}

export type Version = {
    versionType: VersionType;
    ami: string | undefined;
    created: string;
    version_number: string;
};

export enum ProjectState {
    Created = "Created",
    New = "New",
    OpenInProgress = "OpenInProgress",
    Opened = "Opened",
    Closed = "Closed",
}

export type ProjectStateType = {
    type: ProjectState;
}

export type Project = {
    organizationId: string;
    projectId: string;
    name: string;
    state: ProjectStateType;
    packageName: string;
    address: string | null;
    ami: string | null;
    ideVersion: Version | null;
    engineVersion: Version | null;
}