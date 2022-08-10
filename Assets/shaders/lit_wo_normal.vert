out vec4 vWorldPos;
out vec3 vNormal;
out vec2 vUvChannels[maxUvChannelCount];
out mat3 vTangentMatrix;
out mat3 vInvTangentMatrix;

void main()
{
	mat4 boneTransform = 
		bones[int(boneIds[0])] * boneWeights[0] +
		bones[int(boneIds[1])] * boneWeights[1] +
		bones[int(boneIds[2])] * boneWeights[2] +
		bones[int(boneIds[3])] * boneWeights[3];

	vNormal = normalMatrix * mat3(boneTransform) * normal;

	vTangentMatrix[0] = normalize(normalMatrix * mat3(boneTransform) * tangent);
	vTangentMatrix[2] = normalize(vNormal);
	vTangentMatrix[1] = normalize(cross(vTangentMatrix[2], vTangentMatrix[0]));
	vInvTangentMatrix = transpose(vTangentMatrix);

	for (int i = 0; i < maxUvChannelCount; ++i)
	{
		vUvChannels[i] = uvChannels[i];
	}

	vWorldPos = objectMatrix * boneTransform * vec4(position, 1.0f);
	gl_Position = projectionMatrix * viewMatrix * vWorldPos;
}
