import { TimeIcon, EditIcon } from '@chakra-ui/icons'
import {
  Divider,
  HStack,
  IconButton,
  Menu,
  MenuButton,
  MenuItem,
  MenuList,
  Text,
  useColorModeValue
} from '@chakra-ui/react'
import { useState, useEffect, useMemo, useCallback } from 'react'
import { useMacroContext } from '../../../../contexts/macroContext'
import { HIDLookup } from '../../../../constants/HIDmap'
import { mouseEnumLookup } from '../../../../constants/MouseMap'
import { sysEventLookup } from '../../../../constants/SystemEventMap'
import { ActionEventType } from '../../../../types'
import {
  DownArrowIcon,
  DownUpArrowsIcon,
  KebabVertical,
  UpArrowIcon
} from '../../../icons'

interface Props {
  id: number
  element: ActionEventType
  recording: boolean
  stopRecording: () => void
}

export default function SortableItem({
  id,
  element,
  recording,
  stopRecording
}: Props) {
  const [isEditable, setIsEditable] = useState(true)
  const [displayText, setDisplayText] = useState<string | undefined>('')
  const {
    selectedElementId,
    onElementAdd,
    onElementDelete,
    updateSelectedElementId
  } = useMacroContext()
  const bg = useColorModeValue('primary-light.50', 'primary-dark.700')
  const kebabColour = useColorModeValue('primary-light.500', 'primary-dark.500')
  const kebabHoverColour = useColorModeValue(
    'primary-light.900',
    'primary-dark.400'
  )
  const isKeyboardKey = useMemo(() => {
    return element.type === 'KeyPressEventAction'
  }, [element.type])
  const isSelected = selectedElementId === id - 1

  const onDuplicate = useCallback(() => {
    const newElement = { ...element }
    onElementAdd(newElement)
  }, [element, onElementAdd])

  useEffect(() => {
    switch (element.type) {
      case 'KeyPressEventAction':
        setDisplayText(HIDLookup.get(element.data.keypress)?.displayString)
        setIsEditable(true)
        break
      case 'DelayEventAction':
        setDisplayText(element.data.toString() + ' ms')
        setIsEditable(true)
        break
      case 'MouseEventAction':
        setDisplayText(
          mouseEnumLookup.get(element.data.data.button)?.displayString
        )
        setIsEditable(true)
        break
      case 'SystemEventAction':
        switch (element.data.type) {
          case 'Open':
            setDisplayText(
              sysEventLookup.get(element.data.action.type)?.displayString
            )
            setIsEditable(true)
            break
          case 'Volume':
            setDisplayText(
              sysEventLookup.get(element.data.action.type)?.displayString
            )
            setIsEditable(false)
            break
          case 'Clipboard':
            setDisplayText(
              sysEventLookup.get(element.data.action.type)?.displayString
            )
            switch (element.data.action.type) {
              case 'PasteUserDefinedString':
                setIsEditable(true)
                break
              case 'Sarcasm':
                setIsEditable(false)
                break
              default:
                setIsEditable(false)
                break
            }
            break
          default:
            setDisplayText('err')
            setIsEditable(false)
            break
        }
        break
      default:
        break
    }
  }, [element.data, element.type, id])

  const iconToDisplay = useMemo(() => {
    switch (element.type) {
      case 'DelayEventAction':
        return <TimeIcon />
      case 'KeyPressEventAction':
        if (element.data.keytype === 'DownUp') {
          return <DownUpArrowsIcon />
        } else if (element.data.keytype === 'Down') {
          return <DownArrowIcon />
        } else {
          return <UpArrowIcon />
        }
      case 'MouseEventAction':
        if (element.data.data.type === 'DownUp') {
          return <DownUpArrowsIcon />
        } else if (element.data.data.type === 'Down') {
          return <DownArrowIcon />
        } else {
          return <UpArrowIcon />
        }
      case 'SystemEventAction':
        return <></>
      default:
        return <></>
    }
  }, [element])

  const onItemPress = useCallback(() => {
    if (recording) return

    if (isSelected) {
      return
    }
    updateSelectedElementId(id - 1)
  }, [id, isSelected, recording, updateSelectedElementId])

  const onEditButtonPress = useCallback(() => {
    stopRecording()
    if (isSelected) {
      return
    }
    updateSelectedElementId(id - 1)
  }, [id, isSelected, stopRecording, updateSelectedElementId])

  const onDeleteButtonPress = useCallback(() => {
    if (isSelected) {
      updateSelectedElementId(undefined)
    }
    onElementDelete(id - 1)
  }, [id, isSelected, onElementDelete, updateSelectedElementId])

  return (
    <HStack
      w="full"
      h="full"
      justifyContent="space-around"
      spacing="0px"
      onClick={onItemPress}
    >
      <HStack p={1} px={2} w="full" spacing={0} gap={4}>
        {iconToDisplay}
        <Text
          {...(isKeyboardKey
            ? {
                bg: bg,
                border: '1px solid',
                py: 1,
                px: 3
              }
            : {})}
          borderColor={kebabColour}
          fontWeight={
            selectedElementId !== undefined && id === selectedElementId + 1
              ? 'bold'
              : 'normal'
          }
          rounded="md"
        >
          {displayText}
        </Text>
      </HStack>
      <HStack py={2} pr={2} h="full" spacing={0} gap={2} alignItems="flex-end">
        {isEditable && (
          <IconButton
            variant="brandSecondary"
            aria-label="Edit item"
            icon={<EditIcon />}
            size={['xs', 'sm']}
            onClick={onEditButtonPress}
          />
        )}
        <Menu variant="brand">
          <MenuButton
            h="24px"
            aria-label="Item options"
            color={kebabColour}
            px={2}
            _hover={{ color: kebabHoverColour }}
          >
            <KebabVertical />
          </MenuButton>
          <MenuList p="2" right={0}>
            <MenuItem onClick={onDuplicate}>Duplicate</MenuItem>
            <Divider />
            <MenuItem onClick={onDeleteButtonPress} textColor="red.500">
              Delete
            </MenuItem>
          </MenuList>
        </Menu>
      </HStack>
    </HStack>
  )
}
