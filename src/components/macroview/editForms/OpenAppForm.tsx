import { Divider, Text, Textarea } from '@chakra-ui/react'
import { useEffect, useState } from 'react'
import { useMacroContext } from '../../../contexts/macroContext'
import { useSelectedElement } from '../../../contexts/selectors'

export default function OpenAppForm() {
  const [path, setPath] = useState('')
  const selectedElement = useSelectedElement()
  const { selectedElementId, updateElement } = useMacroContext()

  useEffect(() => {
    if (selectedElement === undefined) {
      return
    }

    if (selectedElement.type !== 'SystemEventAction') {
      return
    }
    if (selectedElement.data.type !== 'Open') {
      return
    }

    setPath(selectedElement.data.path)
  }, [selectedElement])

  const onPathChange = (event: any) => {
    if (selectedElement === undefined || selectedElementId === undefined) {
      return
    }
    if (selectedElement.type !== 'SystemEventAction') {
      return
    }
    setPath(event.target.value)
    const temp = { ...selectedElement }
    temp.data = { type: 'Open', path: event.target.value }
    updateElement(temp, selectedElementId)
  }

  return (
    <>
      <Text fontWeight="semibold" fontSize={['sm', 'md']}>
        {'Open Application'}
      </Text>
      <Divider />
      <Text fontSize={['xs', 'sm', 'md']}>Path to application</Text>
      <Textarea value={path} onChange={onPathChange} placeholder="path"/>
    </>
  )
}
